# 6. Data Models

## Event Schema

### Event Enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    New {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
        location: SourceLocation,
    },
    Borrow {
        timestamp: u64,
        borrower_name: String,
        borrower_id: String,
        owner_id: String,
        mutable: bool,
        location: SourceLocation,
    },
    Move {
        timestamp: u64,
        from_id: String,
        to_name: String,
        to_id: String,
        location: SourceLocation,
    },
    Drop {
        timestamp: u64,
        var_id: String,
        location: SourceLocation,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}
```

### Event Types Explained

#### New Event
Triggered when a variable is created.

**Example:**
```rust
let s = String::from("hello");
```

**Event:**
```json
{
  "type": "New",
  "timestamp": 1,
  "var_name": "s",
  "var_id": "s_0x1a2b",
  "type_name": "String",
  "location": {
    "file": "main.rs",
    "line": 5,
    "column": 9
  }
}
```

#### Borrow Event
Triggered when a reference is created.

**Example:**
```rust
let r = &s;
```

**Event:**
```json
{
  "type": "Borrow",
  "timestamp": 2,
  "borrower_name": "r",
  "borrower_id": "r_0x2b3c",
  "owner_id": "s_0x1a2b",
  "mutable": false,
  "location": {
    "file": "main.rs",
    "line": 6,
    "column": 9
  }
}
```

#### Move Event
Triggered when ownership is transferred.

**Example:**
```rust
let t = s;  // s moved to t
```

**Event:**
```json
{
  "type": "Move",
  "timestamp": 3,
  "from_id": "s_0x1a2b",
  "to_name": "t",
  "to_id": "t_0x3c4d",
  "location": {
    "file": "main.rs",
    "line": 7,
    "column": 9
  }
}
```

#### Drop Event
Triggered when a variable goes out of scope.

**Example:**
```rust
}  // s dropped here
```

**Event:**
```json
{
  "type": "Drop",
  "timestamp": 4,
  "var_id": "s_0x1a2b",
  "location": {
    "file": "main.rs",
    "line": 8,
    "column": 1
  }
}
```

---

## Ownership Graph Structure

### Graph Data Model

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct OwnershipGraph {
    pub nodes: Vec<Variable>,
    pub edges: Vec<Relationship>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Variable {
    pub id: String,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
    pub location: SourceLocation,
    pub status: VariableStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VariableStatus {
    Active,
    Moved,
    Dropped,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub kind: RelationshipKind,
    pub from_id: String,
    pub to_id: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RelationshipKind {
    Owns,
    BorrowsImmut,
    BorrowsMut,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub total_events: usize,
    pub total_variables: usize,
    pub max_timestamp: u64,
    pub source_file: String,
    pub function_name: Option<String>,
}
```

### Graph Example

**Code:**
```rust
fn example() {
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("{} {}", r1, r2);
}
```

**Graph:**
```json
{
  "nodes": [
    {
      "id": "s_0x1a2b",
      "name": "s",
      "type_name": "String",
      "created_at": 1,
      "dropped_at": 5,
      "location": { "file": "main.rs", "line": 2, "column": 9 },
      "status": "Dropped"
    },
    {
      "id": "r1_0x2b3c",
      "name": "r1",
      "type_name": "&String",
      "created_at": 2,
      "dropped_at": 4,
      "location": { "file": "main.rs", "line": 3, "column": 9 },
      "status": "Dropped"
    },
    {
      "id": "r2_0x3c4d",
      "name": "r2",
      "type_name": "&String",
      "created_at": 3,
      "dropped_at": 4,
      "location": { "file": "main.rs", "line": 4, "column": 9 },
      "status": "Dropped"
    }
  ],
  "edges": [
    {
      "id": "edge_1",
      "kind": "BorrowsImmut",
      "from_id": "r1_0x2b3c",
      "to_id": "s_0x1a2b",
      "start_time": 2,
      "end_time": 4
    },
    {
      "id": "edge_2",
      "kind": "BorrowsImmut",
      "from_id": "r2_0x3c4d",
      "to_id": "s_0x1a2b",
      "start_time": 3,
      "end_time": 4
    }
  ],
  "metadata": {
    "total_events": 7,
    "total_variables": 3,
    "max_timestamp": 5,
    "source_file": "main.rs",
    "function_name": "example"
  }
}
```

---

## JSON Export Format

### Complete Export Structure

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct BorrowScopeOutput {
    pub version: String,
    pub events: Vec<Event>,
    pub graph: OwnershipGraph,
    pub config: ExportConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    pub timestamp_unit: String,  // "microseconds"
    pub include_stdlib: bool,
    pub max_depth: Option<u32>,
}
```

### Full Example Output

```json
{
  "version": "0.1.0",
  "events": [
    {
      "type": "New",
      "timestamp": 1,
      "var_name": "s",
      "var_id": "s_0x1a2b",
      "type_name": "String",
      "location": { "file": "main.rs", "line": 2, "column": 9 }
    },
    {
      "type": "Borrow",
      "timestamp": 2,
      "borrower_name": "r1",
      "borrower_id": "r1_0x2b3c",
      "owner_id": "s_0x1a2b",
      "mutable": false,
      "location": { "file": "main.rs", "line": 3, "column": 9 }
    },
    {
      "type": "Borrow",
      "timestamp": 3,
      "borrower_name": "r2",
      "borrower_id": "r2_0x3c4d",
      "owner_id": "s_0x1a2b",
      "mutable": false,
      "location": { "file": "main.rs", "line": 4, "column": 9 }
    },
    {
      "type": "Drop",
      "timestamp": 4,
      "var_id": "r2_0x3c4d",
      "location": { "file": "main.rs", "line": 5, "column": 30 }
    },
    {
      "type": "Drop",
      "timestamp": 4,
      "var_id": "r1_0x2b3c",
      "location": { "file": "main.rs", "line": 5, "column": 30 }
    },
    {
      "type": "Drop",
      "timestamp": 5,
      "var_id": "s_0x1a2b",
      "location": { "file": "main.rs", "line": 6, "column": 1 }
    }
  ],
  "graph": {
    "nodes": [
      {
        "id": "s_0x1a2b",
        "name": "s",
        "type_name": "String",
        "created_at": 1,
        "dropped_at": 5,
        "location": { "file": "main.rs", "line": 2, "column": 9 },
        "status": "Dropped"
      },
      {
        "id": "r1_0x2b3c",
        "name": "r1",
        "type_name": "&String",
        "created_at": 2,
        "dropped_at": 4,
        "location": { "file": "main.rs", "line": 3, "column": 9 },
        "status": "Dropped"
      },
      {
        "id": "r2_0x3c4d",
        "name": "r2",
        "type_name": "&String",
        "created_at": 3,
        "dropped_at": 4,
        "location": { "file": "main.rs", "line": 4, "column": 9 },
        "status": "Dropped"
      }
    ],
    "edges": [
      {
        "id": "edge_1",
        "kind": "BorrowsImmut",
        "from_id": "r1_0x2b3c",
        "to_id": "s_0x1a2b",
        "start_time": 2,
        "end_time": 4
      },
      {
        "id": "edge_2",
        "kind": "BorrowsImmut",
        "from_id": "r2_0x3c4d",
        "to_id": "s_0x1a2b",
        "start_time": 3,
        "end_time": 4
      }
    ],
    "metadata": {
      "total_events": 6,
      "total_variables": 3,
      "max_timestamp": 5,
      "source_file": "main.rs",
      "function_name": "example"
    }
  },
  "config": {
    "timestamp_unit": "microseconds",
    "include_stdlib": false,
    "max_depth": null
  }
}
```

---

## Lifetime Representation (Phase 4)

### Extended Data Model

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Lifetime {
    pub name: String,  // 'a, 'b, 'static
    pub start: u64,
    pub end: u64,
    pub variables: Vec<String>,  // var_ids bound to this lifetime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VariableWithLifetime {
    pub id: String,
    pub name: String,
    pub type_name: String,
    pub lifetimes: Vec<String>,  // lifetime names
    pub created_at: u64,
    pub dropped_at: Option<u64>,
}
```

### Example with Lifetimes

**Code:**
```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

**Extended Graph:**
```json
{
  "lifetimes": [
    {
      "name": "'a",
      "start": 1,
      "end": 10,
      "variables": ["x_0x1a2b", "y_0x2b3c", "return_0x3c4d"]
    }
  ],
  "nodes": [
    {
      "id": "x_0x1a2b",
      "name": "x",
      "type_name": "&'a str",
      "lifetimes": ["'a"],
      "created_at": 1,
      "dropped_at": 10
    }
  ]
}
```

---

## Error Representation (Phase 4)

### Borrow Checker Error Model

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct BorrowError {
    pub error_type: BorrowErrorType,
    pub message: String,
    pub primary_location: SourceLocation,
    pub conflicting_borrows: Vec<ConflictingBorrow>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BorrowErrorType {
    MutableBorrowWhileImmutable,
    ImmutableBorrowWhileMutable,
    UseAfterMove,
    UseAfterDrop,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictingBorrow {
    pub var_id: String,
    pub borrow_type: String,  // "immutable" | "mutable"
    pub location: SourceLocation,
}
```

### Error Example

```json
{
  "error_type": "MutableBorrowWhileImmutable",
  "message": "cannot borrow `s` as mutable because it is also borrowed as immutable",
  "primary_location": { "file": "main.rs", "line": 4, "column": 14 },
  "conflicting_borrows": [
    {
      "var_id": "r1_0x2b3c",
      "borrow_type": "immutable",
      "location": { "file": "main.rs", "line": 3, "column": 14 }
    }
  ],
  "suggestion": "Consider dropping the immutable borrow before creating a mutable one"
}
```

---

## Data Flow Summary

```
User Code
    ↓
[Instrumentation]
    ↓
Runtime Events → Event Stream
    ↓
[Graph Builder]
    ↓
OwnershipGraph + Events
    ↓
[Serialization]
    ↓
JSON File
    ↓
[UI Parser]
    ↓
Visualization
```

### File Naming Convention

```
borrowscope-output/
├── example_2024-10-30_23-05-00.json      # Full output
├── example_2024-10-30_23-05-00.events    # Events only
└── example_2024-10-30_23-05-00.graph     # Graph only
```

---

## Schema Versioning

### Version Strategy

```rust
pub const SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
```

**Compatibility rules:**
- Major version change: Breaking changes, UI must update
- Minor version change: New fields (backward compatible)
- Patch version change: Bug fixes, no schema changes

### Migration Support

```rust
pub fn migrate_schema(data: &str, from: &str, to: &str) -> Result<String> {
    // Handle schema migrations between versions
}
```
