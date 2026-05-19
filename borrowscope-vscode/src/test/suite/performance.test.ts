import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("8.6 Performance Profiling and Optimization", () => {
  let perf: any;
  let pkgJson: any;

  before(() => {
    perf = require(path.join(ROOT, "out", "performance"));
    pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
  });

  // === PerformanceMonitor ===
  it("PerformanceMonitor class exists", () => {
    assert.ok(perf.PerformanceMonitor);
    const m = new perf.PerformanceMonitor();
    assert.ok(m);
  });

  it("time() records and returns result", () => {
    const m = new perf.PerformanceMonitor();
    const result = m.time("test", () => 42);
    assert.strictEqual(result, 42);
    assert.ok(m.getStats("test"));
  });

  it("timeAsync() records async operations", async () => {
    const m = new perf.PerformanceMonitor();
    const result = await m.timeAsync("async-test", async () => "hello");
    assert.strictEqual(result, "hello");
    assert.ok(m.getStats("async-test"));
  });

  it("record() stores timing", () => {
    const m = new perf.PerformanceMonitor();
    m.record("manual", 50);
    m.record("manual", 75);
    const stats = m.getStats("manual");
    assert.strictEqual(stats.count, 2);
    assert.strictEqual(stats.avg, 62.5);
    assert.strictEqual(stats.max, 75);
    assert.strictEqual(stats.min, 50);
  });

  it("getStats returns null for unknown label", () => {
    const m = new perf.PerformanceMonitor();
    assert.strictEqual(m.getStats("unknown"), null);
  });

  it("report() generates formatted string", () => {
    const m = new perf.PerformanceMonitor();
    m.record("op1", 10);
    m.record("op2", 20);
    const report = m.report();
    assert.ok(report.includes("Performance Report"));
    assert.ok(report.includes("op1"));
    assert.ok(report.includes("op2"));
  });

  it("report() handles empty state", () => {
    const m = new perf.PerformanceMonitor();
    const report = m.report();
    assert.ok(report.includes("No timings recorded"));
  });

  it("clear() removes all timings", () => {
    const m = new perf.PerformanceMonitor();
    m.record("x", 10);
    m.clear();
    assert.strictEqual(m.getLabels().length, 0);
  });

  it("keeps only last 100 measurements", () => {
    const m = new perf.PerformanceMonitor();
    for (let i = 0; i < 150; i++) m.record("flood", i);
    assert.strictEqual(m.getStats("flood").count, 100);
  });

  // === Debouncer ===
  it("Debouncer class exists", () => {
    assert.ok(perf.Debouncer);
    const d = new perf.Debouncer();
    assert.ok(d);
  });

  it("debounce delays execution", (done) => {
    const d = new perf.Debouncer();
    let called = false;
    d.debounce(() => { called = true; }, 20);
    assert.strictEqual(called, false);
    assert.strictEqual(d.isPending, true);
    setTimeout(() => {
      assert.strictEqual(called, true);
      assert.strictEqual(d.isPending, false);
      done();
    }, 50);
  });

  it("debounce cancels previous call", (done) => {
    const d = new perf.Debouncer();
    let count = 0;
    d.debounce(() => { count++; }, 20);
    d.debounce(() => { count++; }, 20);
    d.debounce(() => { count++; }, 20);
    setTimeout(() => {
      assert.strictEqual(count, 1); // only last one fires
      done();
    }, 50);
  });

  it("cancel() prevents execution", (done) => {
    const d = new perf.Debouncer();
    let called = false;
    d.debounce(() => { called = true; }, 20);
    d.cancel();
    setTimeout(() => {
      assert.strictEqual(called, false);
      done();
    }, 50);
  });

  it("timeSinceLastCall returns -1 initially", () => {
    const d = new perf.Debouncer();
    assert.strictEqual(d.timeSinceLastCall, -1);
  });

  // === isFileTooLarge ===
  it("isFileTooLarge returns false for small files", () => {
    const doc = { lineCount: 500 };
    assert.strictEqual(perf.isFileTooLarge(doc), false);
  });

  it("isFileTooLarge returns true for huge files", () => {
    const doc = { lineCount: 15000 };
    assert.strictEqual(perf.isFileTooLarge(doc), true);
  });

  it("isFileTooLarge respects custom limit", () => {
    const doc = { lineCount: 200 };
    assert.strictEqual(perf.isFileTooLarge(doc, 100), true);
    assert.strictEqual(perf.isFileTooLarge(doc, 500), false);
  });

  // === Command registered ===
  it("showPerformanceReport command in package.json", () => {
    assert.ok(pkgJson.contributes.commands.some((c: any) => c.command === "borrowscope.showPerformanceReport"));
  });

  it("registerPerformanceCommand is a function", () => {
    assert.strictEqual(typeof perf.registerPerformanceCommand, "function");
  });

  // === Integration ===
  it("extension.ts imports performance module", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
    assert.ok(src.includes("PerformanceMonitor"));
    assert.ok(src.includes("registerPerformanceCommand"));
  });
});
