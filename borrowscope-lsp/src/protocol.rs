//! Custom LSP protocol extensions for BorrowScope
//!
//! Defines custom methods beyond standard LSP for ownership visualization.

use serde::{Deserialize, Serialize};

/// Variable ownership information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    pub name: String,
    pub type_name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub is_owner: bool,
}

/// Borrow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowInfo {
    pub name: String,
    pub borrowed_from: String,
    pub is_mutable: bool,
    pub start_line: u32,
    pub end_line: u32,
}

/// Move (ownership transfer) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveInfo {
    pub from: String,
    pub to: String,
    pub line: u32,
}

/// Drop point information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropInfo {
    pub name: String,
    pub line: u32,
    pub is_explicit: bool,
}

/// Borrow conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub description: String,
    pub first_borrow: BorrowInfo,
    pub second_borrow: BorrowInfo,
}

/// Complete ownership analysis for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipInfo {
    pub variables: Vec<VariableInfo>,
    pub borrows: Vec<BorrowInfo>,
    pub moves: Vec<MoveInfo>,
    pub drops: Vec<DropInfo>,
    pub conflicts: Vec<ConflictInfo>,
}

/// Timeline span for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSpan {
    pub variable: String,
    pub kind: SpanKind,
    pub start_line: u32,
    pub end_line: u32,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanKind {
    Owner,
    Borrow,
    BorrowMut,
}

/// Decoration to render in the editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decoration {
    pub start_line: u32,
    pub start_char: u32,
    pub end_line: u32,
    pub end_char: u32,
    pub kind: DecorationKind,
    pub text: Option<String>,
    pub hover_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecorationKind {
    Owner,
    Borrow,
    BorrowMut,
    Move,
    Drop,
    Conflict,
}
