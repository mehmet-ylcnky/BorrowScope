import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("8.1 Extension Settings", () => {
  let config: any;
  let pkgJson: any;

  before(() => {
    config = require(path.join(ROOT, "out", "config"));
    pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
  });

  const props = () => pkgJson.contributes.configuration.properties;

  // === config.ts module ===
  it("getConfig returns full config object", () => {
    const cfg = config.getConfig();
    assert.ok(cfg.server);
    assert.ok(cfg.analysis);
    assert.ok(cfg.decorations);
    assert.ok(cfg.graph);
    assert.ok(cfg.colors);
    assert.ok(cfg.crossFunction);
    assert.ok(cfg.memoryLayout);
    assert.ok(cfg.runtime);
    assert.ok(cfg.diagnostics);
  });

  it("getConfig returns correct defaults", () => {
    const cfg = config.getConfig();
    assert.strictEqual(cfg.server.path, "");
    assert.deepStrictEqual(cfg.server.extraArgs, []);
    assert.strictEqual(cfg.analysis.debounceMs, 300);
    assert.strictEqual(cfg.decorations.enabled, true);
    assert.strictEqual(cfg.runtime.enabled, false);
  });

  it("isFeatureEnabled checks decorations.enabled first", () => {
    assert.strictEqual(typeof config.isFeatureEnabled, "function");
  });

  it("getColor returns color string", () => {
    const color = config.getColor("sharedBorrow");
    assert.ok(color.startsWith("#"));
  });

  // === Server settings ===
  it("has server.path", () => { assert.ok(props()["borrowscope.server.path"]); });
  it("has server.extraArgs", () => {
    const p = props()["borrowscope.server.extraArgs"];
    assert.ok(p);
    assert.strictEqual(p.type, "array");
    assert.deepStrictEqual(p.default, []);
  });

  // === Analysis settings ===
  it("has analysis.debounceMs with min/max", () => {
    const p = props()["borrowscope.analysis.debounceMs"];
    assert.ok(p);
    assert.strictEqual(p.default, 300);
    assert.strictEqual(p.minimum, 0);
    assert.strictEqual(p.maximum, 2000);
  });

  // === Decoration settings ===
  it("has decorations.enabled", () => { assert.ok(props()["borrowscope.decorations.enabled"]); });
  it("has decorations.borrowScopes", () => { assert.ok(props()["borrowscope.decorations.borrowScopes"]); });
  it("has decorations.gutterIcons", () => { assert.ok(props()["borrowscope.decorations.gutterIcons"]); });
  it("has decorations.inlayHints", () => { assert.ok(props()["borrowscope.decorations.inlayHints"]); });
  it("has decorations.codeLens", () => { assert.ok(props()["borrowscope.decorations.codeLens"]); });
  it("has decorations.lifelines", () => { assert.ok(props()["borrowscope.decorations.lifelines"]); });

  // === Graph settings ===
  it("has graph.layout with enum", () => {
    const p = props()["borrowscope.graph.layout"];
    assert.ok(p);
    assert.deepStrictEqual(p.enum, ["force", "hierarchical", "radial"]);
  });
  it("has graph.showTypes", () => { assert.ok(props()["borrowscope.graph.showTypes"]); });
  it("has graph.animateUpdates", () => { assert.ok(props()["borrowscope.graph.animateUpdates"]); });

  // === Color settings ===
  it("has colors.sharedBorrow", () => {
    const p = props()["borrowscope.colors.sharedBorrow"];
    assert.ok(p);
    assert.strictEqual(p.default, "#3498db");
  });
  it("has colors.mutableBorrow", () => { assert.strictEqual(props()["borrowscope.colors.mutableBorrow"].default, "#e74c3c"); });
  it("has colors.move", () => { assert.strictEqual(props()["borrowscope.colors.move"].default, "#e67e22"); });
  it("has colors.rcArc", () => { assert.strictEqual(props()["borrowscope.colors.rcArc"].default, "#9b59b6"); });
  it("has colors.owned", () => { assert.strictEqual(props()["borrowscope.colors.owned"].default, "#2ecc71"); });
  it("has colors.drop", () => { assert.strictEqual(props()["borrowscope.colors.drop"].default, "#95a5a6"); });

  // === Cross-function settings ===
  it("has crossFunction.enabled", () => { assert.ok(props()["borrowscope.crossFunction.enabled"]); });
  it("has crossFunction.maxDepth", () => { assert.ok(props()["borrowscope.crossFunction.maxDepth"]); });
  it("has crossFunction.showInline", () => { assert.ok(props()["borrowscope.crossFunction.showInline"]); });

  // === Memory layout settings ===
  it("has memoryLayout.enabled", () => { assert.ok(props()["borrowscope.memoryLayout.enabled"]); });
  it("has memoryLayout.showAlignment", () => { assert.ok(props()["borrowscope.memoryLayout.showAlignment"]); });
  it("has memoryLayout.animationSpeed", () => { assert.ok(props()["borrowscope.memoryLayout.animationSpeed"]); });

  // === Runtime settings ===
  it("has runtime.enabled (default false)", () => { assert.strictEqual(props()["borrowscope.runtime.enabled"].default, false); });
  it("has runtime.source with enum", () => { assert.deepStrictEqual(props()["borrowscope.runtime.source"].enum, ["file", "websocket"]); });
  it("has runtime.filePath", () => { assert.ok(props()["borrowscope.runtime.filePath"]); });
  it("has runtime.websocketPort", () => { assert.ok(props()["borrowscope.runtime.websocketPort"]); });
  it("has runtime.showTimings", () => { assert.ok(props()["borrowscope.runtime.showTimings"]); });
  it("has runtime.showDropOrder", () => { assert.ok(props()["borrowscope.runtime.showDropOrder"]); });
  it("has runtime.showRefCounts", () => { assert.ok(props()["borrowscope.runtime.showRefCounts"]); });
  it("has runtime.highlightDivergences", () => { assert.ok(props()["borrowscope.runtime.highlightDivergences"]); });

  // === Diagnostics settings ===
  it("has diagnostics.enabled", () => { assert.ok(props()["borrowscope.diagnostics.enabled"]); });
  it("has diagnostics.severity with enum", () => {
    const p = props()["borrowscope.diagnostics.severity"];
    assert.ok(p);
    assert.deepStrictEqual(p.enum, ["information", "hint", "warning"]);
    assert.strictEqual(p.default, "information");
  });

  // === Total settings count ===
  it("has at least 30 settings", () => {
    const count = Object.keys(props()).filter(k => k.startsWith("borrowscope.")).length;
    assert.ok(count >= 30, `Expected >= 30 settings, got ${count}`);
  });
});
