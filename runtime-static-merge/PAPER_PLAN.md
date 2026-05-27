# Merging Static and Runtime Analysis for Rust Ownership Visualization

## Paper Plan - Sections and Subsections

---

## Abstract

---

## 1. Introduction

### 1.1 The Two Worlds of Ownership Analysis
### 1.2 Why Neither Alone is Sufficient
### 1.3 The Merge Hypothesis
### 1.4 Contributions

---

## 2. Background

### 2.1 Static Ownership Analysis (borrowscope-lsp)
### 2.2 Runtime Event Tracking (borrowscope-runtime)
### 2.3 The 88 Event Type Taxonomy
### 2.4 Related Work (Miri, Valgrind, Sanitizers, KLEE)

---

## 3. Event Ingestion

### 3.1 File-Based Ingestion (JSON Watcher)
### 3.2 WebSocket Live Streaming
### 3.3 Event Validation and Parsing
### 3.4 Internally-Tagged vs Externally-Tagged Format Handling

---

## 4. Variable Mapping

### 4.1 The Correlation Problem (Runtime IDs vs Static Declarations)
### 4.2 Name + Line + File Matching Strategy
### 4.3 Handling Shadowed Variables
### 4.4 Unmapped Variables (runtime_only and static_only)

---

## 5. The Merge Algorithm

### 5.1 MergedVariable Construction
### 5.2 RuntimeInfo Aggregation from Event Streams
### 5.3 Drop Order Computation
### 5.4 Agreement Classification (match, diverge, runtime_only, static_only)
### 5.5 Merge Summary Statistics

---

## 6. Divergence Detection

### 6.1 Detection Architecture (Per-Variable Analysis)
### 6.2 Rc/Arc Leak Detection (rc_leak)
### 6.3 Reference Cycle Detection (rc_cycle)
### 6.4 Missing Drop Detection (missing_drop)
### 6.5 Async Borrow Held Across Await (async_borrow_held)
### 6.6 Unsafe Hidden Behavior (unsafe_hidden)
### 6.7 Conditional Move Detection (conditional_move)
### 6.8 Weak Upgrade Failure (weak_upgrade_fail)
### 6.9 Channel Receive Failure (channel_recv_fail)
### 6.10 Use After Move (use_after_move)
### 6.11 Severity Classification and Actionable Suggestions

---

## 7. Visualization of Merged Data

### 7.1 Inline Runtime Decorations (Timing, Drop Order, Divergence Highlights)
### 7.2 Runtime View: Timeline Sub-tab
### 7.3 Runtime View: Drop Order Sub-tab
### 7.4 Runtime View: Reference Count Sub-tab
### 7.5 Runtime View: Event Log Sub-tab
### 7.6 Status Bar Integration (Event Count, Divergence Badge)

---

## 8. Async Borrow Tracking

### 8.1 The Problem: Borrows Held Across Await Points
### 8.2 Detecting AwaitStart Events During Active Borrows
### 8.3 Duration Measurement and Future Identification
### 8.4 Implications for Send Trait Compliance

---

## 9. Evaluation

### 9.1 Merge Performance (Events per Second)
### 9.2 Divergence Detection Accuracy
### 9.3 False Positive Analysis
### 9.4 Real-World Divergence Examples

---

## 10. Limitations and Future Work

### 10.1 Single Execution Path Limitation
### 10.2 Thread Interleaving Sensitivity
### 10.3 Planned: Multi-Run Aggregation
### 10.4 Planned: Confidence Scoring for Divergences
### 10.5 Planned: Integration with borrowscope-macro for Automated Instrumentation

---

## 11. Conclusion
