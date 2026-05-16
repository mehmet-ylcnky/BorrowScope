import * as assert from "assert";
import * as path from "path";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("12.7 Rc/Arc Reference Count Timeline", () => {
  let rct: any;

  before(() => {
    rct = require(path.join(ROOT, "out", "refcount-timeline"));
  });

  // === buildRefCountTimeline ===

  it("returns null for unknown var_id", () => {
    const result = rct.buildRefCountTimeline([], "unknown");
    assert.strictEqual(result, null);
  });

  it("builds timeline for RcNew", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc1", var_id: "rc1_1", type_name: "i32", strong_count: 1, weak_count: 0 } },
    ];
    const t = rct.buildRefCountTimeline(events, "rc1_1");
    assert.ok(t);
    assert.strictEqual(t.var_name, "rc1");
    assert.strictEqual(t.is_arc, false);
    assert.strictEqual(t.points.length, 1);
    assert.strictEqual(t.points[0].event, "new");
    assert.strictEqual(t.points[0].strong_count, 1);
  });

  it("builds timeline for ArcNew", () => {
    const events = [
      { ArcNew: { timestamp: 10, var_name: "a1", var_id: "a1_1", type_name: "Mutex<i32>", strong_count: 1, weak_count: 0 } },
    ];
    const t = rct.buildRefCountTimeline(events, "a1_1");
    assert.ok(t);
    assert.strictEqual(t.is_arc, true);
    assert.strictEqual(t.type_name, "Mutex<i32>");
  });

  it("tracks RcClone increasing strong count", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc1", var_id: "rc1_1", type_name: "i32", strong_count: 1, weak_count: 0 } },
      { RcClone: { timestamp: 20, var_name: "rc2", var_id: "rc2_1", source_id: "rc1_1", strong_count: 2, weak_count: 0 } },
      { RcClone: { timestamp: 30, var_name: "rc3", var_id: "rc3_1", source_id: "rc1_1", strong_count: 3, weak_count: 0 } },
    ];
    const t = rct.buildRefCountTimeline(events, "rc1_1");
    assert.strictEqual(t.points.length, 3);
    assert.strictEqual(t.points[1].event, "clone");
    assert.strictEqual(t.points[1].strong_count, 2);
    assert.strictEqual(t.points[2].strong_count, 3);
    assert.strictEqual(t.peak_strong, 3);
    assert.strictEqual(t.total_clones, 2);
  });

  it("tracks ArcClone", () => {
    const events = [
      { ArcNew: { timestamp: 10, var_name: "a1", var_id: "a1_1", type_name: "String", strong_count: 1, weak_count: 0 } },
      { ArcClone: { timestamp: 20, var_name: "a2", var_id: "a2_1", source_id: "a1_1", strong_count: 2, weak_count: 0 } },
    ];
    const t = rct.buildRefCountTimeline(events, "a1_1");
    assert.strictEqual(t.total_clones, 1);
    assert.strictEqual(t.peak_strong, 2);
  });

  it("tracks Drop decreasing strong count", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc1", var_id: "rc1_1", type_name: "i32", strong_count: 1, weak_count: 0 } },
      { RcClone: { timestamp: 20, var_name: "rc2", var_id: "rc2_1", source_id: "rc1_1", strong_count: 2, weak_count: 0 } },
      { Drop: { timestamp: 30, var_id: "rc1_1" } },
    ];
    const t = rct.buildRefCountTimeline(events, "rc1_1");
    assert.strictEqual(t.points.length, 3);
    assert.strictEqual(t.points[2].event, "drop");
    assert.strictEqual(t.points[2].strong_count, 1);
    assert.strictEqual(t.total_drops, 1);
    assert.strictEqual(t.final_strong, 1);
  });

  it("tracks WeakNew", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc1", var_id: "rc1_1", type_name: "i32", strong_count: 1, weak_count: 0 } },
      { WeakNew: { timestamp: 20, var_name: "w1", var_id: "w1_1", source_id: "rc1_1", weak_count: 1, location: "src/main.rs:5:5" } },
    ];
    const t = rct.buildRefCountTimeline(events, "rc1_1");
    assert.strictEqual(t.points.length, 2);
    assert.strictEqual(t.points[1].event, "weak_new");
    assert.strictEqual(t.points[1].weak_count, 1);
    assert.strictEqual(t.peak_weak, 1);
  });

  it("tracks WeakClone", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc1", var_id: "rc1_1", type_name: "i32", strong_count: 1, weak_count: 0 } },
      { WeakNew: { timestamp: 20, var_name: "w1", var_id: "w1_1", source_id: "rc1_1", weak_count: 1, location: "src/main.rs:5:5" } },
      { WeakClone: { timestamp: 30, var_name: "w2", var_id: "w2_1", source_id: "rc1_1", weak_count: 2, location: "src/main.rs:6:5" } },
    ];
    const t = rct.buildRefCountTimeline(events, "rc1_1");
    assert.strictEqual(t.points[2].event, "weak_clone");
    assert.strictEqual(t.peak_weak, 2);
  });

  it("tracks WeakUpgrade success", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc1", var_id: "rc1_1", type_name: "i32", strong_count: 1, weak_count: 0 } },
      { WeakUpgrade: { timestamp: 20, weak_id: "rc1_1", success: true, location: "src/main.rs:8:5" } },
    ];
    const t = rct.buildRefCountTimeline(events, "rc1_1");
    assert.strictEqual(t.points[1].event, "upgrade_ok");
  });

  it("tracks WeakUpgrade failure", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc1", var_id: "rc1_1", type_name: "i32", strong_count: 1, weak_count: 0 } },
      { Drop: { timestamp: 20, var_id: "rc1_1" } },
      { WeakUpgrade: { timestamp: 30, weak_id: "rc1_1", success: false, location: "src/main.rs:10:5" } },
    ];
    const t = rct.buildRefCountTimeline(events, "rc1_1");
    assert.strictEqual(t.points[2].event, "upgrade_fail");
  });

  it("detects leak (final_strong > 0 after drops)", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc1", var_id: "rc1_1", type_name: "Node", strong_count: 1, weak_count: 0 } },
      { RcClone: { timestamp: 20, var_name: "rc2", var_id: "rc2_1", source_id: "rc1_1", strong_count: 2, weak_count: 0 } },
      { Drop: { timestamp: 30, var_id: "rc1_1" } },
      // rc2 never dropped — leaked
    ];
    const t = rct.buildRefCountTimeline(events, "rc1_1");
    assert.strictEqual(t.final_strong, 1);
    assert.strictEqual(t.leaked, true);
  });

  it("not leaked if all refs dropped to 0", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc1", var_id: "rc1_1", type_name: "i32", strong_count: 1, weak_count: 0 } },
      { Drop: { timestamp: 20, var_id: "rc1_1" } },
    ];
    const t = rct.buildRefCountTimeline(events, "rc1_1");
    assert.strictEqual(t.final_strong, 0);
    assert.strictEqual(t.leaked, false);
  });

  // === buildAllRefCountTimelines ===

  it("builds timelines for all Rc/Arc vars", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc1", var_id: "rc1_1", type_name: "i32", strong_count: 1, weak_count: 0 } },
      { ArcNew: { timestamp: 20, var_name: "a1", var_id: "a1_1", type_name: "String", strong_count: 1, weak_count: 0 } },
      { RcClone: { timestamp: 30, var_name: "rc2", var_id: "rc2_1", source_id: "rc1_1", strong_count: 2, weak_count: 0 } },
    ];
    const timelines = rct.buildAllRefCountTimelines(events);
    assert.strictEqual(timelines.length, 2);
  });

  it("returns empty for no Rc/Arc events", () => {
    const events = [{ New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } }];
    const timelines = rct.buildAllRefCountTimelines(events);
    assert.strictEqual(timelines.length, 0);
  });

  // === refCountAnnotation ===

  it("annotation for new", () => {
    const text = rct.refCountAnnotation({ strong_count: 1, weak_count: 0, event: "new" });
    assert.ok(text.includes("refs: 1"));
  });

  it("annotation for clone shows ↑", () => {
    const text = rct.refCountAnnotation({ strong_count: 3, weak_count: 0, event: "clone" });
    assert.ok(text.includes("↑"));
    assert.ok(text.includes("refs: 3"));
  });

  it("annotation for drop shows ↓", () => {
    const text = rct.refCountAnnotation({ strong_count: 1, weak_count: 0, event: "drop" });
    assert.ok(text.includes("↓"));
  });

  it("annotation includes weak count when > 0", () => {
    const text = rct.refCountAnnotation({ strong_count: 2, weak_count: 3, event: "clone" });
    assert.ok(text.includes("weak: 3"));
  });

  // === detectRefCountIssues ===

  it("detects leak issue", () => {
    const timelines = [{
      var_id: "rc1_1", var_name: "rc1", type_name: "Node", is_arc: false,
      points: [], peak_strong: 3, peak_weak: 0, final_strong: 2, final_weak: 0,
      leaked: true, total_clones: 2, total_drops: 1,
    }];
    const issues = rct.detectRefCountIssues(timelines);
    assert.ok(issues.some((i: any) => i.kind === "leak"));
  });

  it("detects high refcount issue", () => {
    const timelines = [{
      var_id: "rc1_1", var_name: "rc1", type_name: "Data", is_arc: false,
      points: [], peak_strong: 15, peak_weak: 0, final_strong: 0, final_weak: 0,
      leaked: false, total_clones: 14, total_drops: 15,
    }];
    const issues = rct.detectRefCountIssues(timelines);
    assert.ok(issues.some((i: any) => i.kind === "high_refcount"));
  });

  it("detects dangling weak issue", () => {
    const timelines = [{
      var_id: "rc1_1", var_name: "rc1", type_name: "i32", is_arc: false,
      points: [{ event: "upgrade_fail", strong_count: 0, weak_count: 1, timestamp: 100, line: 10, variable: "w1" }],
      peak_strong: 1, peak_weak: 1, final_strong: 0, final_weak: 1,
      leaked: false, total_clones: 0, total_drops: 1,
    }];
    const issues = rct.detectRefCountIssues(timelines);
    assert.ok(issues.some((i: any) => i.kind === "dangling_weak"));
  });

  it("no issues for healthy Rc", () => {
    const timelines = [{
      var_id: "rc1_1", var_name: "rc1", type_name: "i32", is_arc: false,
      points: [], peak_strong: 3, peak_weak: 0, final_strong: 0, final_weak: 0,
      leaked: false, total_clones: 2, total_drops: 3,
    }];
    const issues = rct.detectRefCountIssues(timelines);
    assert.strictEqual(issues.length, 0);
  });
});
