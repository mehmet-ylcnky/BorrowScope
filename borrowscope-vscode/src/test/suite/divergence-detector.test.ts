import * as assert from "assert";
import * as path from "path";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("12.6 Divergence Detection", () => {
  let detector: any;

  before(() => {
    detector = require(path.join(ROOT, "out", "divergence-detector"));
  });

  function makeMerged(overrides: any): any {
    return {
      name: "x", var_id: "x_1", line: 3, file: "src/main.rs",
      static_info: { type_display: "i32", ownership_category: "Owned", is_copy: false },
      runtime_info: {
        actual_lifetime_ns: 1000, actual_borrow_count: 0, actual_mut_borrow_count: 0,
        was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 1010,
        ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0,
        thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2,
      },
      agreement: "match", divergences: [],
      ...overrides,
    };
  }

  // === rc_leak ===

  it("detects Rc leak (never dropped, ref_count > 0)", () => {
    const m = makeMerged({
      static_info: { type_display: "Rc<i32>", ownership_category: "Rc", is_copy: false },
      runtime_info: { actual_lifetime_ns: -1, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: -1, drop_timestamp: -1, ref_count_peak: 2, ref_count_final: 2, weak_count_peak: 0, clone_count: 1, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(divs.some((d: any) => d.kind === "rc_leak"));
    assert.ok(divs.find((d: any) => d.kind === "rc_leak").severity === "error");
  });

  it("no rc_leak if Rc is properly dropped", () => {
    const m = makeMerged({
      static_info: { type_display: "Rc<i32>", ownership_category: "Rc", is_copy: false },
      runtime_info: { actual_lifetime_ns: 1000, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 1010, ref_count_peak: 3, ref_count_final: 0, weak_count_peak: 0, clone_count: 2, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 4 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(!divs.some((d: any) => d.kind === "rc_leak"));
  });

  // === rc_cycle ===

  it("detects Rc cycle (multiple clones, never freed)", () => {
    const m = makeMerged({
      static_info: { type_display: "Rc<Node>", ownership_category: "Rc", is_copy: false },
      runtime_info: { actual_lifetime_ns: -1, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: -1, drop_timestamp: -1, ref_count_peak: 3, ref_count_final: 2, weak_count_peak: 0, clone_count: 2, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 3 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(divs.some((d: any) => d.kind === "rc_cycle"));
  });

  // === missing_drop ===

  it("detects missing drop for non-Copy owned var", () => {
    const m = makeMerged({
      runtime_info: { actual_lifetime_ns: -1, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: -1, drop_timestamp: -1, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 1 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(divs.some((d: any) => d.kind === "missing_drop"));
    assert.ok(divs.find((d: any) => d.kind === "missing_drop").severity === "warning");
  });

  it("no missing_drop for Copy types", () => {
    const m = makeMerged({
      static_info: { type_display: "i32", ownership_category: "Owned", is_copy: true },
      runtime_info: { actual_lifetime_ns: -1, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: -1, drop_timestamp: -1, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 1 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(!divs.some((d: any) => d.kind === "missing_drop"));
  });

  it("no missing_drop if moved", () => {
    const m = makeMerged({
      runtime_info: { actual_lifetime_ns: -1, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: true, move_destination: "y", drop_order: -1, drop_timestamp: -1, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(!divs.some((d: any) => d.kind === "missing_drop"));
  });

  // === async_borrow_held ===

  it("detects borrow held across await", () => {
    const m = makeMerged({
      runtime_info: { actual_lifetime_ns: 5000, actual_borrow_count: 1, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 5010, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [{ await_line: 8, duration_ns: 2000000, future_name: "fetch" }], unsafe_accesses: 0, event_count: 4 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(divs.some((d: any) => d.kind === "async_borrow_held"));
    const d = divs.find((d: any) => d.kind === "async_borrow_held");
    assert.ok(d.description.includes("1 await"));
    assert.ok(d.runtime_evidence.includes("fetch"));
  });

  it("no async_borrow_held if no crossings", () => {
    const m = makeMerged({});
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(!divs.some((d: any) => d.kind === "async_borrow_held"));
  });

  // === unsafe_hidden ===

  it("detects unsafe hidden for non-RawPtr var", () => {
    const m = makeMerged({
      runtime_info: { actual_lifetime_ns: 1000, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 1010, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 3, event_count: 5 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(divs.some((d: any) => d.kind === "unsafe_hidden"));
    assert.ok(divs.find((d: any) => d.kind === "unsafe_hidden").severity === "info");
  });

  it("no unsafe_hidden for RawPtr category", () => {
    const m = makeMerged({
      static_info: { type_display: "*const i32", ownership_category: "RawPtr", is_copy: false },
      runtime_info: { actual_lifetime_ns: 1000, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 1010, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 5, event_count: 6 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(!divs.some((d: any) => d.kind === "unsafe_hidden"));
  });

  // === conditional_move ===

  it("detects conditional move (owned, not moved, no borrows)", () => {
    const m = makeMerged({
      runtime_info: { actual_lifetime_ns: 1000, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 1010, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(divs.some((d: any) => d.kind === "conditional_move"));
    assert.ok(divs.find((d: any) => d.kind === "conditional_move").severity === "info");
  });

  it("no conditional_move if var was borrowed", () => {
    const m = makeMerged({
      runtime_info: { actual_lifetime_ns: 1000, actual_borrow_count: 2, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 1010, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 4 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.ok(!divs.some((d: any) => d.kind === "conditional_move"));
  });

  // === weak_upgrade_fail ===

  it("detects Weak::upgrade failure", () => {
    const m = makeMerged({ var_id: "w_1", name: "w",
      static_info: { type_display: "Weak<i32>", ownership_category: "Weak", is_copy: false },
      runtime_info: { actual_lifetime_ns: 1000, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 1010, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 1, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 3 },
    });
    const events = [
      { WeakUpgrade: { timestamp: 500, weak_id: "w_1", success: false, location: "src/main.rs:10:5" } },
    ];
    const divs = detector.detectAllDivergences(m, events);
    assert.ok(divs.some((d: any) => d.kind === "weak_upgrade_fail"));
    assert.ok(divs.find((d: any) => d.kind === "weak_upgrade_fail").severity === "warning");
  });

  it("no weak_upgrade_fail if all upgrades succeed", () => {
    const m = makeMerged({ var_id: "w_1",
      static_info: { type_display: "Weak<i32>", ownership_category: "Weak", is_copy: false },
    });
    const events = [
      { WeakUpgrade: { timestamp: 500, weak_id: "w_1", success: true, location: "src/main.rs:10:5" } },
    ];
    const divs = detector.detectAllDivergences(m, events);
    assert.ok(!divs.some((d: any) => d.kind === "weak_upgrade_fail"));
  });

  // === channel_recv_fail ===

  it("detects channel receive failure", () => {
    const m = makeMerged({ var_id: "r_1", name: "receiver",
      static_info: { type_display: "Receiver<i32>", ownership_category: "Owned", is_copy: false },
    });
    const events = [
      { ChannelRecv: { timestamp: 500, receiver_id: "r_1", success: false, location: "src/main.rs:15:5" } },
    ];
    const divs = detector.detectAllDivergences(m, events);
    assert.ok(divs.some((d: any) => d.kind === "channel_recv_fail"));
  });

  // === use_after_move ===

  it("detects use after move", () => {
    const m = makeMerged({
      var_id: "x_1",
      runtime_info: { actual_lifetime_ns: -1, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: true, move_destination: "y", drop_order: -1, drop_timestamp: -1, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 3 },
      events: [
        { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "String" } },
        { Move: { timestamp: 50, from_id: "x_1", to_name: "y", to_id: "y_1" } },
        { Borrow: { timestamp: 60, borrower_name: "r", borrower_id: "r_1", owner_id: "x_1", mutable: false } },
      ],
    });
    const allEvents = [
      { Move: { timestamp: 50, from_id: "x_1", to_name: "y", to_id: "y_1" } },
    ];
    const divs = detector.detectAllDivergences(m, allEvents);
    assert.ok(divs.some((d: any) => d.kind === "use_after_move"));
    assert.ok(divs.find((d: any) => d.kind === "use_after_move").severity === "error");
  });

  // === No divergences for healthy variable ===

  it("no divergences for properly used and dropped variable", () => {
    const m = makeMerged({
      static_info: { type_display: "String", ownership_category: "Owned", is_copy: false },
      runtime_info: { actual_lifetime_ns: 1000, actual_borrow_count: 2, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 1010, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 4 },
    });
    const divs = detector.detectAllDivergences(m, []);
    assert.strictEqual(divs.length, 0);
  });

  // === No runtime_info ===

  it("returns empty for no runtime_info", () => {
    const m = makeMerged({ runtime_info: null });
    const divs = detector.detectAllDivergences(m, []);
    assert.strictEqual(divs.length, 0);
  });

  // === analyzeDivergences ===

  it("analyzeDivergences summarizes across all variables", () => {
    const merged = [
      makeMerged({ name: "a", runtime_info: { actual_lifetime_ns: -1, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: -1, drop_timestamp: -1, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 1 } }),
      makeMerged({ name: "b", static_info: { type_display: "Rc<i32>", ownership_category: "Rc", is_copy: false }, runtime_info: { actual_lifetime_ns: -1, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: -1, drop_timestamp: -1, ref_count_peak: 2, ref_count_final: 2, weak_count_peak: 0, clone_count: 1, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 } }),
    ];
    const analysis = detector.analyzeDivergences(merged, []);
    assert.ok(analysis.total > 0);
    assert.ok(analysis.variables_with_divergences.length > 0);
    assert.ok(analysis.by_severity.error > 0 || analysis.by_severity.warning > 0);
  });

  // === DetailedDivergence has suggestion ===

  it("all divergences have suggestions", () => {
    const m = makeMerged({
      static_info: { type_display: "Rc<Node>", ownership_category: "Rc", is_copy: false },
      runtime_info: { actual_lifetime_ns: -1, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: -1, drop_timestamp: -1, ref_count_peak: 3, ref_count_final: 2, weak_count_peak: 0, clone_count: 2, thread_id: null, await_crossings: [{ await_line: 5, duration_ns: 1000, future_name: "f" }], unsafe_accesses: 1, event_count: 5 },
    });
    const divs = detector.detectAllDivergences(m, []);
    for (const d of divs) {
      assert.ok(d.suggestion, `Divergence "${d.kind}" should have a suggestion`);
      assert.ok(d.runtime_evidence, `Divergence "${d.kind}" should have runtime_evidence`);
    }
  });
});
