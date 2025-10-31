# Section 71: Detecting Borrow Conflicts

## Learning Objectives

By the end of this section, you will:
- Understand Rust's borrowing rules
- Detect simultaneous mutable borrows
- Detect mutable + immutable conflicts
- Implement conflict detection algorithms
- Report conflicts with context

## Prerequisites

- Section 70 (Graph Traversal Algorithms)
- Deep understanding of Rust's borrow checker rules
- Familiarity with time-based analysis

---

## Rust's Borrowing Rules

**Rule 1:** Multiple immutable borrows are allowed  
**Rule 2:** Only one mutable borrow at a time  
**Rule 3:** No immutable borrows while a mutable borrow exists

```rust
// Valid
let x = 42;
let r1 = &x;
let r2 = &x;

// Invalid: multiple mutable borrows
let mut x = 42;
let r1 = &mut x;
let r2 = &mut x;  // ERROR

// Invalid: mutable + immutable
let mut x = 42;
let r1 = &x;
let r2 = &mut x;  // ERROR
```

---

## Conflict Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    MultipleMutableBorrows,
    MutableWithImmutable,
}

#[derive(Debug, Clone)]
pub struct BorrowConflict {
    pub conflict_type: ConflictType,
    pub owner_id: usize,
    pub borrowers: Vec<usize>,
    pub time_range: (u64, u64),
}
```

---

## Detection Algorithm

### Step 1: Find Active Borrows at Each Time

```rust
impl OwnershipGraph {
    pub fn active_borrows_at(&self, owner_id: usize, time: u64) -> Vec<(usize, bool)> {
        let owner_node = match self.id_to_node.get(&owner_id) {
            Some(&n) => n,
            None => return vec![],
        };
        
        self.graph.edges_directed(owner_node, Direction::Incoming)
            .filter_map(|edge| {
                let borrower_node = edge.source();
                let borrower = self.graph.node_weight(borrower_node)?;
                
                // Check if borrow is active at this time
                let (borrow_time, is_mut) = match edge.weight() {
                    Relationship::BorrowsImmut { at } => (*at, false),
                    Relationship::BorrowsMut { at } => (*at, true),
                    _ => return None,
                };
                
                if borrow_time <= time && borrower.dropped_at.map_or(true, |d| d > time) {
                    Some((borrower.id, is_mut))
                } else {
                    None
                }
            })
            .collect()
    }
}
```

### Step 2: Check for Conflicts

```rust
impl OwnershipGraph {
    pub fn check_conflicts_at(&self, owner_id: usize, time: u64) -> Option<BorrowConflict> {
        let active = self.active_borrows_at(owner_id, time);
        
        if active.is_empty() {
            return None;
        }
        
        let mut mutable_borrows = vec![];
        let mut immutable_borrows = vec![];
        
        for (borrower_id, is_mut) in active {
            if is_mut {
                mutable_borrows.push(borrower_id);
            } else {
                immutable_borrows.push(borrower_id);
            }
        }
        
        // Check for multiple mutable borrows
        if mutable_borrows.len() > 1 {
            return Some(BorrowConflict {
                conflict_type: ConflictType::MultipleMutableBorrows,
                owner_id,
                borrowers: mutable_borrows,
                time_range: (time, time),
            });
        }
        
        // Check for mutable + immutable
        if !mutable_borrows.is_empty() && !immutable_borrows.is_empty() {
            let mut all_borrowers = mutable_borrows.clone();
            all_borrowers.extend(immutable_borrows);
            
            return Some(BorrowConflict {
                conflict_type: ConflictType::MutableWithImmutable,
                owner_id,
                borrowers: all_borrowers,
                time_range: (time, time),
            });
        }
        
        None
    }
}
```

### Step 3: Scan All Times

```rust
impl OwnershipGraph {
    pub fn find_all_conflicts(&self) -> Vec<BorrowConflict> {
        let mut conflicts = vec![];
        let mut all_times = std::collections::BTreeSet::new();
        
        // Collect all relevant timestamps
        for var in self.graph.node_weights() {
            all_times.insert(var.created_at);
            if let Some(dropped) = var.dropped_at {
                all_times.insert(dropped);
            }
        }
        
        for edge in self.graph.edge_references() {
            let time = match edge.weight() {
                Relationship::BorrowsImmut { at } => *at,
                Relationship::BorrowsMut { at } => *at,
                _ => continue,
            };
            all_times.insert(time);
        }
        
        // Check each variable at each time
        for var in self.graph.node_weights() {
            for &time in &all_times {
                if let Some(conflict) = self.check_conflicts_at(var.id, time) {
                    // Avoid duplicates
                    if !conflicts.iter().any(|c: &BorrowConflict| {
                        c.owner_id == conflict.owner_id && 
                        c.borrowers == conflict.borrowers
                    }) {
                        conflicts.push(conflict);
                    }
                }
            }
        }
        
        conflicts
    }
}
```

---

## Optimized Detection

### Time Range Analysis

```rust
#[derive(Debug, Clone)]
struct BorrowInterval {
    borrower_id: usize,
    is_mut: bool,
    start: u64,
    end: u64,
}

impl OwnershipGraph {
    fn get_borrow_intervals(&self, owner_id: usize) -> Vec<BorrowInterval> {
        let owner_node = match self.id_to_node.get(&owner_id) {
            Some(&n) => n,
            None => return vec![],
        };
        
        self.graph.edges_directed(owner_node, Direction::Incoming)
            .filter_map(|edge| {
                let borrower_node = edge.source();
                let borrower = self.graph.node_weight(borrower_node)?;
                
                let (start, is_mut) = match edge.weight() {
                    Relationship::BorrowsImmut { at } => (*at, false),
                    Relationship::BorrowsMut { at } => (*at, true),
                    _ => return None,
                };
                
                let end = borrower.dropped_at.unwrap_or(u64::MAX);
                
                Some(BorrowInterval {
                    borrower_id: borrower.id,
                    is_mut,
                    start,
                    end,
                })
            })
            .collect()
    }
    
    pub fn find_conflicts_optimized(&self) -> Vec<BorrowConflict> {
        let mut conflicts = vec![];
        
        for var in self.graph.node_weights() {
            let intervals = self.get_borrow_intervals(var.id);
            
            // Check for overlapping intervals
            for i in 0..intervals.len() {
                for j in (i + 1)..intervals.len() {
                    let a = &intervals[i];
                    let b = &intervals[j];
                    
                    // Check if intervals overlap
                    let overlap_start = a.start.max(b.start);
                    let overlap_end = a.end.min(b.end);
                    
                    if overlap_start < overlap_end {
                        // Intervals overlap
                        if a.is_mut || b.is_mut {
                            // Conflict: at least one is mutable
                            let conflict_type = if a.is_mut && b.is_mut {
                                ConflictType::MultipleMutableBorrows
                            } else {
                                ConflictType::MutableWithImmutable
                            };
                            
                            conflicts.push(BorrowConflict {
                                conflict_type,
                                owner_id: var.id,
                                borrowers: vec![a.borrower_id, b.borrower_id],
                                time_range: (overlap_start, overlap_end),
                            });
                        }
                    }
                }
            }
        }
        
        conflicts
    }
}
```

---

## Conflict Reporting

```rust
impl BorrowConflict {
    pub fn format(&self, graph: &OwnershipGraph) -> String {
        let owner = graph.get_variable(self.owner_id)
            .map(|v| v.name.as_str())
            .unwrap_or("<unknown>");
        
        let borrower_names: Vec<_> = self.borrowers.iter()
            .filter_map(|&id| graph.get_variable(id))
            .map(|v| v.name.as_str())
            .collect();
        
        match self.conflict_type {
            ConflictType::MultipleMutableBorrows => {
                format!(
                    "Multiple mutable borrows of '{}' by: {}",
                    owner,
                    borrower_names.join(", ")
                )
            }
            ConflictType::MutableWithImmutable => {
                format!(
                    "Mutable and immutable borrows of '{}' by: {}",
                    owner,
                    borrower_names.join(", ")
                )
            }
        }
    }
}

impl OwnershipGraph {
    pub fn report_conflicts(&self) -> String {
        let conflicts = self.find_conflicts_optimized();
        
        if conflicts.is_empty() {
            return "No borrow conflicts detected.".to_string();
        }
        
        let mut report = format!("Found {} conflict(s):\n\n", conflicts.len());
        
        for (i, conflict) in conflicts.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, conflict.format(self)));
            report.push_str(&format!("   Time range: {} - {}\n\n", 
                conflict.time_range.0, conflict.time_range.1));
        }
        
        report
    }
}
```

---

## Example Usage

```rust
#[test]
fn test_multiple_mutable_borrows() {
    let mut graph = OwnershipGraph::new();
    
    // let mut x = 42;
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });
    
    // let r1 = &mut x;
    graph.add_variable(Variable {
        id: 2,
        name: "r1".into(),
        type_name: "&mut i32".into(),
        created_at: 1100,
        dropped_at: Some(1300),
        scope_depth: 0,
    });
    graph.add_borrow(2, 1, true, 1100);
    
    // let r2 = &mut x;  // Conflict!
    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 1200,
        dropped_at: Some(1400),
        scope_depth: 0,
    });
    graph.add_borrow(3, 1, true, 1200);
    
    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].conflict_type, ConflictType::MultipleMutableBorrows);
}

#[test]
fn test_mutable_with_immutable() {
    let mut graph = OwnershipGraph::new();
    
    graph.add_variable(Variable {
        id: 1, name: "x".into(), type_name: "i32".into(),
        created_at: 1000, dropped_at: None, scope_depth: 0,
    });
    
    // let r1 = &x;
    graph.add_variable(Variable {
        id: 2, name: "r1".into(), type_name: "&i32".into(),
        created_at: 1100, dropped_at: Some(1300), scope_depth: 0,
    });
    graph.add_borrow(2, 1, false, 1100);
    
    // let r2 = &mut x;  // Conflict!
    graph.add_variable(Variable {
        id: 3, name: "r2".into(), type_name: "&mut i32".into(),
        created_at: 1200, dropped_at: Some(1400), scope_depth: 0,
    });
    graph.add_borrow(3, 1, true, 1200);
    
    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].conflict_type, ConflictType::MutableWithImmutable);
}

#[test]
fn test_no_conflict_sequential() {
    let mut graph = OwnershipGraph::new();
    
    graph.add_variable(Variable {
        id: 1, name: "x".into(), type_name: "i32".into(),
        created_at: 1000, dropped_at: None, scope_depth: 0,
    });
    
    // let r1 = &mut x;
    graph.add_variable(Variable {
        id: 2, name: "r1".into(), type_name: "&mut i32".into(),
        created_at: 1100, dropped_at: Some(1200), scope_depth: 0,
    });
    graph.add_borrow(2, 1, true, 1100);
    
    // let r2 = &mut x;  // OK: r1 already dropped
    graph.add_variable(Variable {
        id: 3, name: "r2".into(), type_name: "&mut i32".into(),
        created_at: 1300, dropped_at: Some(1400), scope_depth: 0,
    });
    graph.add_borrow(3, 1, true, 1300);
    
    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}
```

---

## Limitations

**Note:** Our runtime tracking can only detect conflicts that actually occur at runtime. The Rust compiler prevents most conflicts at compile time.

**What we can detect:**
- RefCell borrow violations (runtime checks)
- Logical errors in our tracking
- Educational examples

**What we cannot detect:**
- Compile-time prevented conflicts (they never run)
- Conflicts in code paths not executed

---

## Integration with UI

```rust
#[derive(Serialize)]
pub struct ConflictReport {
    pub conflicts: Vec<ConflictInfo>,
}

#[derive(Serialize)]
pub struct ConflictInfo {
    pub conflict_type: String,
    pub owner: String,
    pub borrowers: Vec<String>,
    pub time_range: (u64, u64),
}

impl OwnershipGraph {
    pub fn export_conflicts(&self) -> ConflictReport {
        let conflicts = self.find_conflicts_optimized();
        
        ConflictReport {
            conflicts: conflicts.iter().map(|c| {
                ConflictInfo {
                    conflict_type: format!("{:?}", c.conflict_type),
                    owner: self.get_variable(c.owner_id)
                        .map(|v| v.name.clone())
                        .unwrap_or_else(|| "<unknown>".into()),
                    borrowers: c.borrowers.iter()
                        .filter_map(|&id| self.get_variable(id))
                        .map(|v| v.name.clone())
                        .collect(),
                    time_range: c.time_range,
                }
            }).collect(),
        }
    }
}
```

---

## Key Takeaways

✅ **Borrowing rules** - Multiple immutable OR one mutable  
✅ **Conflict types** - Multiple mutable, mutable + immutable  
✅ **Time-based analysis** - Check overlapping borrow intervals  
✅ **Optimized detection** - Interval overlap algorithm  
✅ **Reporting** - Format conflicts for users  

---

## Further Reading

- [Rust borrow checker](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [Non-lexical lifetimes](https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html#non-lexical-lifetimes)

---

**Previous:** [70-graph-traversal-algorithms.md](./70-graph-traversal-algorithms.md)  
**Next:** [72-graph-serialization.md](./72-graph-serialization.md)

**Progress:** 6/10 ⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜
