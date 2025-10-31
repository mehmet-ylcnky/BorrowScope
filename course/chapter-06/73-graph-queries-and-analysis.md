# Section 73: Graph Queries and Analysis

## Learning Objectives

By the end of this section, you will:
- Implement powerful query APIs
- Analyze ownership patterns
- Compute graph statistics
- Build a query DSL
- Optimize query performance

## Prerequisites

- Section 72 (Graph Serialization)
- Understanding of iterator patterns
- Familiarity with graph algorithms

---

## Query API Design

```rust
pub struct QueryBuilder<'a> {
    graph: &'a OwnershipGraph,
}

impl OwnershipGraph {
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder { graph: self }
    }
}
```

---

## Basic Queries

### Find Variable by Name

```rust
impl<'a> QueryBuilder<'a> {
    pub fn find_by_name(&self, name: &str) -> Option<&Variable> {
        self.graph.graph.node_weights()
            .find(|v| v.name == name)
    }
    
    pub fn find_all_by_name(&self, name: &str) -> Vec<&Variable> {
        self.graph.graph.node_weights()
            .filter(|v| v.name == name)
            .collect()
    }
}
```

### Find by Type

```rust
impl<'a> QueryBuilder<'a> {
    pub fn find_by_type(&self, type_name: &str) -> Vec<&Variable> {
        self.graph.graph.node_weights()
            .filter(|v| v.type_name == type_name)
            .collect()
    }
    
    pub fn find_references(&self) -> Vec<&Variable> {
        self.graph.graph.node_weights()
            .filter(|v| v.type_name.starts_with('&'))
            .collect()
    }
    
    pub fn find_mutable_references(&self) -> Vec<&Variable> {
        self.graph.graph.node_weights()
            .filter(|v| v.type_name.starts_with("&mut"))
            .collect()
    }
}
```

### Alive at Time

```rust
impl<'a> QueryBuilder<'a> {
    pub fn alive_at(&self, time: u64) -> Vec<&Variable> {
        self.graph.graph.node_weights()
            .filter(|v| {
                v.created_at <= time && v.dropped_at.map_or(true, |d| d > time)
            })
            .collect()
    }
    
    pub fn created_between(&self, start: u64, end: u64) -> Vec<&Variable> {
        self.graph.graph.node_weights()
            .filter(|v| v.created_at >= start && v.created_at <= end)
            .collect()
    }
}
```

---

## Relationship Queries

### Direct Borrows

```rust
impl<'a> QueryBuilder<'a> {
    pub fn direct_borrowers(&self, id: usize) -> Vec<(&Variable, &Relationship)> {
        let node = match self.graph.id_to_node.get(&id) {
            Some(&n) => n,
            None => return vec![],
        };
        
        self.graph.graph.edges_directed(node, Direction::Incoming)
            .filter_map(|edge| {
                let borrower = self.graph.graph.node_weight(edge.source())?;
                Some((borrower, edge.weight()))
            })
            .collect()
    }
    
    pub fn direct_borrows_from(&self, id: usize) -> Vec<(&Variable, &Relationship)> {
        let node = match self.graph.id_to_node.get(&id) {
            Some(&n) => n,
            None => return vec![],
        };
        
        self.graph.graph.edges(node)
            .filter_map(|edge| {
                let target = self.graph.graph.node_weight(edge.target())?;
                Some((target, edge.weight()))
            })
            .collect()
    }
}
```

### Transitive Borrows

```rust
impl<'a> QueryBuilder<'a> {
    pub fn all_borrowers(&self, id: usize) -> Vec<&Variable> {
        use petgraph::visit::Dfs;
        
        let node = match self.graph.id_to_node.get(&id) {
            Some(&n) => n,
            None => return vec![],
        };
        
        let mut dfs = Dfs::new(&self.graph.graph, node);
        let mut borrowers = Vec::new();
        
        while let Some(n) = dfs.next(&self.graph.graph) {
            if n != node {
                if let Some(var) = self.graph.graph.node_weight(n) {
                    borrowers.push(var);
                }
            }
        }
        
        borrowers
    }
    
    pub fn borrow_chain(&self, from_id: usize, to_id: usize) -> Option<Vec<&Variable>> {
        self.graph.shortest_path(from_id, to_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|&id| self.graph.get_variable(id))
                    .collect()
            })
    }
}
```

---

## Statistical Analysis

```rust
#[derive(Debug)]
pub struct GraphStats {
    pub total_variables: usize,
    pub alive_variables: usize,
    pub total_borrows: usize,
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
    pub moves: usize,
    pub avg_lifetime: f64,
    pub max_borrow_depth: usize,
}

impl OwnershipGraph {
    pub fn statistics(&self) -> GraphStats {
        let total_variables = self.graph.node_count();
        let alive_variables = self.graph.node_weights()
            .filter(|v| v.dropped_at.is_none())
            .count();
        
        let mut immutable_borrows = 0;
        let mut mutable_borrows = 0;
        let mut moves = 0;
        
        for edge in self.graph.edge_references() {
            match edge.weight() {
                Relationship::BorrowsImmut { .. } => immutable_borrows += 1,
                Relationship::BorrowsMut { .. } => mutable_borrows += 1,
                Relationship::Moves { .. } => moves += 1,
                _ => {}
            }
        }
        
        let total_borrows = immutable_borrows + mutable_borrows;
        
        let lifetimes: Vec<u64> = self.graph.node_weights()
            .filter_map(|v| {
                v.dropped_at.map(|d| d - v.created_at)
            })
            .collect();
        
        let avg_lifetime = if lifetimes.is_empty() {
            0.0
        } else {
            lifetimes.iter().sum::<u64>() as f64 / lifetimes.len() as f64
        };
        
        let max_borrow_depth = self.graph.node_weights()
            .map(|v| self.query().borrow_depth(v.id))
            .max()
            .unwrap_or(0);
        
        GraphStats {
            total_variables,
            alive_variables,
            total_borrows,
            immutable_borrows,
            mutable_borrows,
            moves,
            avg_lifetime,
            max_borrow_depth,
        }
    }
}

impl<'a> QueryBuilder<'a> {
    fn borrow_depth(&self, id: usize) -> usize {
        let node = match self.graph.id_to_node.get(&id) {
            Some(&n) => n,
            None => return 0,
        };
        
        let mut max_depth = 0;
        let mut stack = vec![(node, 0)];
        let mut visited = std::collections::HashSet::new();
        
        while let Some((current, depth)) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            
            max_depth = max_depth.max(depth);
            
            for neighbor in self.graph.graph.neighbors(current) {
                stack.push((neighbor, depth + 1));
            }
        }
        
        max_depth
    }
}
```

---

## Pattern Detection

### Detect Common Patterns

```rust
#[derive(Debug)]
pub enum OwnershipPattern {
    SimpleOwnership,        // Single owner, no borrows
    SharedBorrow,           // Multiple immutable borrows
    ExclusiveBorrow,        // Single mutable borrow
    BorrowChain,            // Nested borrows (r1 -> r2 -> x)
    RcSharing,              // Rc clones
    ArcSharing,             // Arc clones
}

impl<'a> QueryBuilder<'a> {
    pub fn detect_pattern(&self, id: usize) -> OwnershipPattern {
        let borrowers = self.direct_borrowers(id);
        
        if borrowers.is_empty() {
            return OwnershipPattern::SimpleOwnership;
        }
        
        let has_mut = borrowers.iter().any(|(_, rel)| {
            matches!(rel, Relationship::BorrowsMut { .. })
        });
        
        let has_immut = borrowers.iter().any(|(_, rel)| {
            matches!(rel, Relationship::BorrowsImmut { .. })
        });
        
        let has_rc = borrowers.iter().any(|(_, rel)| {
            matches!(rel, Relationship::RcClone { .. })
        });
        
        let has_arc = borrowers.iter().any(|(_, rel)| {
            matches!(rel, Relationship::ArcClone { .. })
        });
        
        if has_rc {
            return OwnershipPattern::RcSharing;
        }
        
        if has_arc {
            return OwnershipPattern::ArcSharing;
        }
        
        if has_mut && !has_immut && borrowers.len() == 1 {
            return OwnershipPattern::ExclusiveBorrow;
        }
        
        if has_immut && !has_mut {
            return OwnershipPattern::SharedBorrow;
        }
        
        // Check for borrow chain
        if borrowers.len() == 1 {
            let borrower_id = borrowers[0].0.id;
            if !self.direct_borrowers(borrower_id).is_empty() {
                return OwnershipPattern::BorrowChain;
            }
        }
        
        OwnershipPattern::SimpleOwnership
    }
}
```

---

## Advanced Queries

### Find Longest Lifetime

```rust
impl<'a> QueryBuilder<'a> {
    pub fn longest_lived(&self) -> Option<&Variable> {
        self.graph.graph.node_weights()
            .filter_map(|v| {
                v.dropped_at.map(|d| (v, d - v.created_at))
            })
            .max_by_key(|(_, lifetime)| *lifetime)
            .map(|(v, _)| v)
    }
}
```

### Find Most Borrowed

```rust
impl<'a> QueryBuilder<'a> {
    pub fn most_borrowed(&self) -> Option<(&Variable, usize)> {
        self.graph.graph.node_weights()
            .map(|v| {
                let count = self.direct_borrowers(v.id).len();
                (v, count)
            })
            .max_by_key(|(_, count)| *count)
    }
}
```

### Find Orphaned Variables

```rust
impl<'a> QueryBuilder<'a> {
    pub fn orphaned(&self) -> Vec<&Variable> {
        self.graph.graph.node_indices()
            .filter_map(|node| {
                let in_degree = self.graph.graph.neighbors_directed(node, Direction::Incoming).count();
                let out_degree = self.graph.graph.neighbors(node).count();
                
                if in_degree == 0 && out_degree == 0 {
                    self.graph.graph.node_weight(node)
                } else {
                    None
                }
            })
            .collect()
    }
}
```

---

## Query DSL

```rust
pub struct Query<'a> {
    graph: &'a OwnershipGraph,
    filters: Vec<Box<dyn Fn(&Variable) -> bool + 'a>>,
}

impl<'a> QueryBuilder<'a> {
    pub fn filter(self) -> Query<'a> {
        Query {
            graph: self.graph,
            filters: vec![],
        }
    }
}

impl<'a> Query<'a> {
    pub fn name_contains(mut self, pattern: &'a str) -> Self {
        self.filters.push(Box::new(move |v| v.name.contains(pattern)));
        self
    }
    
    pub fn type_is(mut self, type_name: &'a str) -> Self {
        self.filters.push(Box::new(move |v| v.type_name == type_name));
        self
    }
    
    pub fn alive_at(mut self, time: u64) -> Self {
        self.filters.push(Box::new(move |v| {
            v.created_at <= time && v.dropped_at.map_or(true, |d| d > time)
        }));
        self
    }
    
    pub fn scope_depth(mut self, depth: usize) -> Self {
        self.filters.push(Box::new(move |v| v.scope_depth == depth));
        self
    }
    
    pub fn execute(self) -> Vec<&'a Variable> {
        self.graph.graph.node_weights()
            .filter(|v| self.filters.iter().all(|f| f(v)))
            .collect()
    }
}
```

**Usage:**
```rust
let results = graph.query()
    .filter()
    .name_contains("temp")
    .alive_at(1500)
    .scope_depth(2)
    .execute();
```

---

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_by_name() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 1, name: "x".into(), type_name: "i32".into(),
            created_at: 1000, dropped_at: None, scope_depth: 0,
        });
        
        let result = graph.query().find_by_name("x");
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, 1);
    }

    #[test]
    fn test_statistics() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 1, name: "x".into(), type_name: "i32".into(),
            created_at: 1000, dropped_at: Some(2000), scope_depth: 0,
        });
        graph.add_variable(Variable {
            id: 2, name: "r".into(), type_name: "&i32".into(),
            created_at: 1100, dropped_at: Some(1900), scope_depth: 0,
        });
        graph.add_borrow(2, 1, false, 1100);
        
        let stats = graph.statistics();
        assert_eq!(stats.total_variables, 2);
        assert_eq!(stats.immutable_borrows, 1);
        assert_eq!(stats.avg_lifetime, 850.0);
    }

    #[test]
    fn test_query_dsl() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 1, name: "temp_x".into(), type_name: "i32".into(),
            created_at: 1000, dropped_at: None, scope_depth: 0,
        });
        graph.add_variable(Variable {
            id: 2, name: "y".into(), type_name: "i32".into(),
            created_at: 1100, dropped_at: None, scope_depth: 0,
        });
        
        let results = graph.query()
            .filter()
            .name_contains("temp")
            .execute();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "temp_x");
    }
}
```

---

## Performance Optimization

### Caching

```rust
use std::cell::RefCell;

pub struct CachedGraph {
    graph: OwnershipGraph,
    stats_cache: RefCell<Option<GraphStats>>,
}

impl CachedGraph {
    pub fn new(graph: OwnershipGraph) -> Self {
        Self {
            graph,
            stats_cache: RefCell::new(None),
        }
    }
    
    pub fn statistics(&self) -> GraphStats {
        if let Some(stats) = self.stats_cache.borrow().as_ref() {
            return stats.clone();
        }
        
        let stats = self.graph.statistics();
        *self.stats_cache.borrow_mut() = Some(stats.clone());
        stats
    }
    
    pub fn invalidate_cache(&self) {
        *self.stats_cache.borrow_mut() = None;
    }
}
```

---

## Key Takeaways

✅ **Query API** - Flexible interface for graph exploration  
✅ **Statistics** - Compute metrics about ownership patterns  
✅ **Pattern detection** - Identify common ownership idioms  
✅ **Query DSL** - Chainable filters for complex queries  
✅ **Performance** - Cache expensive computations  

---

## Further Reading

- [Iterator patterns in Rust](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [Builder pattern](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)

---

**Previous:** [72-graph-serialization.md](./72-graph-serialization.md)  
**Next:** [74-graph-visualization-data-format.md](./74-graph-visualization-data-format.md)

**Progress:** 8/10 ⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜
