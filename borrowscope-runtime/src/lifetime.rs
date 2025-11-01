//! Lifetime tracking and inference
//!
//! This module provides utilities for tracking and inferring lifetime relationships
//! from runtime events. Since lifetimes are a compile-time concept, we approximate
//! them by tracking scope boundaries and borrow relationships.

use crate::event::Event;
use serde::{Deserialize, Serialize};

/// Represents a lifetime relationship between a borrower and borrowed variable
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifetimeRelation {
    /// ID of the borrowing variable
    pub borrower_id: String,
    /// ID of the borrowed variable
    pub borrowed_id: String,
    /// Timestamp when borrow started
    pub start_time: u64,
    /// Timestamp when borrow ended (None if still active)
    pub end_time: Option<u64>,
    /// Whether this is a mutable borrow
    pub is_mutable: bool,
}

impl LifetimeRelation {
    /// Create a new lifetime relation
    pub fn new(
        borrower_id: String,
        borrowed_id: String,
        start_time: u64,
        is_mutable: bool,
    ) -> Self {
        Self {
            borrower_id,
            borrowed_id,
            start_time,
            end_time: None,
            is_mutable,
        }
    }

    /// Check if this lifetime is still active
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }

    /// Get the duration of this lifetime (if ended)
    pub fn duration(&self) -> Option<u64> {
        self.end_time.map(|end| end - self.start_time)
    }

    /// Check if this lifetime overlaps with another
    pub fn overlaps_with(&self, other: &LifetimeRelation) -> bool {
        let self_end = self.end_time.unwrap_or(u64::MAX);
        let other_end = other.end_time.unwrap_or(u64::MAX);

        self.start_time < other_end && other.start_time < self_end
    }
}

/// Timeline representation for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// All lifetime relations in chronological order
    pub relations: Vec<LifetimeRelation>,
    /// Minimum timestamp
    pub min_time: u64,
    /// Maximum timestamp
    pub max_time: u64,
}

impl Timeline {
    /// Create a timeline from events
    pub fn from_events(events: &[Event]) -> Self {
        let mut relations = Vec::new();
        let mut active_borrows: std::collections::HashMap<String, (String, u64, bool)> =
            std::collections::HashMap::new();

        let mut min_time = u64::MAX;
        let mut max_time = 0;

        for event in events {
            let timestamp = event.timestamp();
            min_time = min_time.min(timestamp);
            max_time = max_time.max(timestamp);

            match event {
                Event::Borrow {
                    borrower_id,
                    owner_id,
                    mutable,
                    timestamp,
                    ..
                } => {
                    active_borrows.insert(
                        borrower_id.clone(),
                        (owner_id.clone(), *timestamp, *mutable),
                    );
                }
                Event::Drop { var_id, timestamp } => {
                    if let Some((borrowed_id, start_time, is_mutable)) =
                        active_borrows.remove(var_id)
                    {
                        let mut relation = LifetimeRelation::new(
                            var_id.clone(),
                            borrowed_id,
                            start_time,
                            is_mutable,
                        );
                        relation.end_time = Some(*timestamp);
                        relations.push(relation);
                    }
                }
                _ => {}
            }
        }

        // Add any still-active borrows
        for (borrower_id, (borrowed_id, start_time, is_mutable)) in active_borrows {
            relations.push(LifetimeRelation::new(
                borrower_id,
                borrowed_id,
                start_time,
                is_mutable,
            ));
        }

        Self {
            relations,
            min_time: if min_time == u64::MAX { 0 } else { min_time },
            max_time,
        }
    }

    /// Get all relations for a specific variable
    pub fn relations_for(&self, var_id: &str) -> Vec<&LifetimeRelation> {
        self.relations
            .iter()
            .filter(|r| r.borrower_id == var_id || r.borrowed_id == var_id)
            .collect()
    }

    /// Get all active relations at a specific timestamp
    pub fn active_at(&self, timestamp: u64) -> Vec<&LifetimeRelation> {
        self.relations
            .iter()
            .filter(|r| r.start_time <= timestamp && r.end_time.map_or(true, |end| timestamp < end))
            .collect()
    }

    /// Check if two variables have overlapping lifetimes
    pub fn lifetimes_overlap(&self, var1: &str, var2: &str) -> bool {
        let relations1 = self.relations_for(var1);
        let relations2 = self.relations_for(var2);

        for r1 in &relations1 {
            for r2 in &relations2 {
                if r1.overlaps_with(r2) {
                    return true;
                }
            }
        }
        false
    }

    /// Get the total duration of the timeline
    pub fn total_duration(&self) -> u64 {
        self.max_time.saturating_sub(self.min_time)
    }
}

/// Lifetime elision rules (for documentation and analysis)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElisionRule {
    /// Each input parameter gets its own lifetime
    EachInputOwn,
    /// If one input, output gets that lifetime
    SingleInputToOutput,
    /// If multiple inputs with &self, output gets self's lifetime
    SelfToOutput,
}

impl ElisionRule {
    /// Get a description of the elision rule
    pub fn description(&self) -> &'static str {
        match self {
            ElisionRule::EachInputOwn => "Each input parameter gets its own lifetime",
            ElisionRule::SingleInputToOutput => "If one input parameter, output gets that lifetime",
            ElisionRule::SelfToOutput => {
                "If multiple inputs with &self, output gets self's lifetime"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifetime_relation_creation() {
        let relation = LifetimeRelation::new("r".to_string(), "x".to_string(), 100, false);

        assert_eq!(relation.borrower_id, "r");
        assert_eq!(relation.borrowed_id, "x");
        assert_eq!(relation.start_time, 100);
        assert_eq!(relation.end_time, None);
        assert!(!relation.is_mutable);
        assert!(relation.is_active());
    }

    #[test]
    fn test_lifetime_relation_duration() {
        let mut relation = LifetimeRelation::new("r".to_string(), "x".to_string(), 100, false);

        assert_eq!(relation.duration(), None);

        relation.end_time = Some(150);
        assert_eq!(relation.duration(), Some(50));
        assert!(!relation.is_active());
    }

    #[test]
    fn test_lifetime_overlap() {
        let r1 = LifetimeRelation {
            borrower_id: "r1".to_string(),
            borrowed_id: "x".to_string(),
            start_time: 100,
            end_time: Some(200),
            is_mutable: false,
        };

        let r2 = LifetimeRelation {
            borrower_id: "r2".to_string(),
            borrowed_id: "x".to_string(),
            start_time: 150,
            end_time: Some(250),
            is_mutable: false,
        };

        let r3 = LifetimeRelation {
            borrower_id: "r3".to_string(),
            borrowed_id: "x".to_string(),
            start_time: 300,
            end_time: Some(400),
            is_mutable: false,
        };

        assert!(r1.overlaps_with(&r2));
        assert!(r2.overlaps_with(&r1));
        assert!(!r1.overlaps_with(&r3));
        assert!(!r3.overlaps_with(&r1));
    }

    #[test]
    fn test_timeline_from_events() {
        let events = vec![
            Event::New {
                timestamp: 0,
                var_name: "x".to_string(),
                var_id: "x_0".to_string(),
                type_name: "i32".to_string(),
            },
            Event::Borrow {
                timestamp: 10,
                borrower_name: "r1".to_string(),
                borrower_id: "r1_0".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Borrow {
                timestamp: 20,
                borrower_name: "r2".to_string(),
                borrower_id: "r2_0".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Drop {
                timestamp: 30,
                var_id: "r2_0".to_string(),
            },
            Event::Drop {
                timestamp: 40,
                var_id: "r1_0".to_string(),
            },
            Event::Drop {
                timestamp: 50,
                var_id: "x_0".to_string(),
            },
        ];

        let timeline = Timeline::from_events(&events);

        assert_eq!(timeline.relations.len(), 2);
        assert_eq!(timeline.min_time, 0);
        assert_eq!(timeline.max_time, 50);
        assert_eq!(timeline.total_duration(), 50);

        // Check r1 relation
        let r1_relation = timeline
            .relations
            .iter()
            .find(|r| r.borrower_id == "r1_0")
            .unwrap();
        assert_eq!(r1_relation.borrowed_id, "x_0");
        assert_eq!(r1_relation.start_time, 10);
        assert_eq!(r1_relation.end_time, Some(40));
        assert_eq!(r1_relation.duration(), Some(30));

        // Check r2 relation
        let r2_relation = timeline
            .relations
            .iter()
            .find(|r| r.borrower_id == "r2_0")
            .unwrap();
        assert_eq!(r2_relation.borrowed_id, "x_0");
        assert_eq!(r2_relation.start_time, 20);
        assert_eq!(r2_relation.end_time, Some(30));
        assert_eq!(r2_relation.duration(), Some(10));
    }

    #[test]
    fn test_timeline_relations_for() {
        let events = vec![
            Event::New {
                timestamp: 0,
                var_name: "x".to_string(),
                var_id: "x_0".to_string(),
                type_name: "i32".to_string(),
            },
            Event::Borrow {
                timestamp: 10,
                borrower_name: "r".to_string(),
                borrower_id: "r_0".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Drop {
                timestamp: 20,
                var_id: "r_0".to_string(),
            },
        ];

        let timeline = Timeline::from_events(&events);

        let x_relations = timeline.relations_for("x_0");
        assert_eq!(x_relations.len(), 1);
        assert_eq!(x_relations[0].borrowed_id, "x_0");

        let r_relations = timeline.relations_for("r_0");
        assert_eq!(r_relations.len(), 1);
        assert_eq!(r_relations[0].borrower_id, "r_0");
    }

    #[test]
    fn test_timeline_active_at() {
        let events = vec![
            Event::Borrow {
                timestamp: 10,
                borrower_name: "r1".to_string(),
                borrower_id: "r1_0".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Borrow {
                timestamp: 20,
                borrower_name: "r2".to_string(),
                borrower_id: "r2_0".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Drop {
                timestamp: 30,
                var_id: "r1_0".to_string(),
            },
        ];

        let timeline = Timeline::from_events(&events);

        // At time 15, only r1 is active
        let active_at_15 = timeline.active_at(15);
        assert_eq!(active_at_15.len(), 1);
        assert_eq!(active_at_15[0].borrower_id, "r1_0");

        // At time 25, both r1 and r2 are active
        let active_at_25 = timeline.active_at(25);
        assert_eq!(active_at_25.len(), 2);

        // At time 35, only r2 is active (r1 dropped at 30)
        let active_at_35 = timeline.active_at(35);
        assert_eq!(active_at_35.len(), 1);
        assert_eq!(active_at_35[0].borrower_id, "r2_0");
    }

    #[test]
    fn test_timeline_lifetimes_overlap() {
        let events = vec![
            Event::Borrow {
                timestamp: 10,
                borrower_name: "r1".to_string(),
                borrower_id: "r1_0".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Borrow {
                timestamp: 20,
                borrower_name: "r2".to_string(),
                borrower_id: "r2_0".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Drop {
                timestamp: 30,
                var_id: "r1_0".to_string(),
            },
            Event::Drop {
                timestamp: 40,
                var_id: "r2_0".to_string(),
            },
        ];

        let timeline = Timeline::from_events(&events);

        // r1 and r2 overlap (r1: 10-30, r2: 20-40)
        assert!(timeline.lifetimes_overlap("r1_0", "r2_0"));
    }

    #[test]
    fn test_mutable_borrow_tracking() {
        let events = vec![
            Event::Borrow {
                timestamp: 10,
                borrower_name: "r".to_string(),
                borrower_id: "r_0".to_string(),
                owner_id: "x_0".to_string(),
                mutable: true,
            },
            Event::Drop {
                timestamp: 20,
                var_id: "r_0".to_string(),
            },
        ];

        let timeline = Timeline::from_events(&events);

        assert_eq!(timeline.relations.len(), 1);
        assert!(timeline.relations[0].is_mutable);
    }

    #[test]
    fn test_elision_rule_descriptions() {
        assert!(!ElisionRule::EachInputOwn.description().is_empty());
        assert!(!ElisionRule::SingleInputToOutput.description().is_empty());
        assert!(!ElisionRule::SelfToOutput.description().is_empty());
    }

    #[test]
    fn test_empty_timeline() {
        let timeline = Timeline::from_events(&[]);

        assert_eq!(timeline.relations.len(), 0);
        assert_eq!(timeline.min_time, 0);
        assert_eq!(timeline.max_time, 0);
        assert_eq!(timeline.total_duration(), 0);
    }

    #[test]
    fn test_nested_lifetimes() {
        let events = vec![
            Event::Borrow {
                timestamp: 10,
                borrower_name: "r1".to_string(),
                borrower_id: "r1_0".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Borrow {
                timestamp: 15,
                borrower_name: "r2".to_string(),
                borrower_id: "r2_0".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Drop {
                timestamp: 20,
                var_id: "r2_0".to_string(),
            },
            Event::Drop {
                timestamp: 25,
                var_id: "r1_0".to_string(),
            },
        ];

        let timeline = Timeline::from_events(&events);

        // r2's lifetime (15-20) is nested within r1's lifetime (10-25)
        let r1 = timeline
            .relations
            .iter()
            .find(|r| r.borrower_id == "r1_0")
            .unwrap();
        let r2 = timeline
            .relations
            .iter()
            .find(|r| r.borrower_id == "r2_0")
            .unwrap();

        assert!(r1.start_time < r2.start_time);
        assert!(r1.end_time.unwrap() > r2.end_time.unwrap());
        assert!(r1.overlaps_with(r2));
    }
}
