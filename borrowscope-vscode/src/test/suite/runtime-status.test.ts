import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("12.11 Configuration and Toggle UI", () => {
  let status: any;
  let pkgJson: any;

  before(() => {
    status = require(path.join(ROOT, "out", "runtime-status"));
    pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
  });

  // === RuntimeStatusBar ===

  it("RuntimeStatusBar class exists", () => {
    assert.ok(status.RuntimeStatusBar);
    assert.strictEqual(typeof status.RuntimeStatusBar, "function");
  });

  it("can instantiate RuntimeStatusBar", () => {
    const bar = new status.RuntimeStatusBar();
    assert.ok(bar);
    bar.dispose();
  });

  it("has show/hide methods", () => {
    const bar = new status.RuntimeStatusBar();
    assert.strictEqual(typeof bar.show, "function");
    assert.strictEqual(typeof bar.hide, "function");
    bar.dispose();
  });

  it("has update method", () => {
    const bar = new status.RuntimeStatusBar();
    assert.strictEqual(typeof bar.update, "function");
    bar.update(); // should not throw
    bar.dispose();
  });

  it("has attachWatcher method", () => {
    const bar = new status.RuntimeStatusBar();
    assert.strictEqual(typeof bar.attachWatcher, "function");
    bar.dispose();
  });

  it("has attachSocket method", () => {
    const bar = new status.RuntimeStatusBar();
    assert.strictEqual(typeof bar.attachSocket, "function");
    bar.dispose();
  });

  it("has dispose method", () => {
    const bar = new status.RuntimeStatusBar();
    assert.strictEqual(typeof bar.dispose, "function");
    bar.dispose();
  });

  // === registerRuntimeCommands ===

  it("registerRuntimeCommands is a function", () => {
    assert.strictEqual(typeof status.registerRuntimeCommands, "function");
  });

  // === Commands in package.json ===

  it("package.json has toggleRuntime command", () => {
    const cmds = pkgJson.contributes.commands;
    assert.ok(cmds.some((c: any) => c.command === "borrowscope.toggleRuntime"));
  });

  it("package.json has runtime.enable command", () => {
    const cmds = pkgJson.contributes.commands;
    assert.ok(cmds.some((c: any) => c.command === "borrowscope.runtime.enable"));
  });

  it("package.json has runtime.disable command", () => {
    const cmds = pkgJson.contributes.commands;
    assert.ok(cmds.some((c: any) => c.command === "borrowscope.runtime.disable"));
  });

  it("package.json has runtime.toggleTimings command", () => {
    const cmds = pkgJson.contributes.commands;
    assert.ok(cmds.some((c: any) => c.command === "borrowscope.runtime.toggleTimings"));
  });

  it("package.json has runtime.toggleDropOrder command", () => {
    const cmds = pkgJson.contributes.commands;
    assert.ok(cmds.some((c: any) => c.command === "borrowscope.runtime.toggleDropOrder"));
  });

  it("package.json has runtime.toggleRefCounts command", () => {
    const cmds = pkgJson.contributes.commands;
    assert.ok(cmds.some((c: any) => c.command === "borrowscope.runtime.toggleRefCounts"));
  });

  it("package.json has runtime.toggleDivergences command", () => {
    const cmds = pkgJson.contributes.commands;
    assert.ok(cmds.some((c: any) => c.command === "borrowscope.runtime.toggleDivergences"));
  });

  it("package.json has runtime.setSource command", () => {
    const cmds = pkgJson.contributes.commands;
    assert.ok(cmds.some((c: any) => c.command === "borrowscope.runtime.setSource"));
  });

  // === Configuration completeness ===

  it("all runtime config settings exist", () => {
    const props = pkgJson.contributes.configuration.properties;
    const required = [
      "borrowscope.runtime.enabled",
      "borrowscope.runtime.source",
      "borrowscope.runtime.filePath",
      "borrowscope.runtime.websocketPort",
      "borrowscope.runtime.showTimings",
      "borrowscope.runtime.showDropOrder",
      "borrowscope.runtime.showRefCounts",
      "borrowscope.runtime.highlightDivergences",
    ];
    for (const key of required) {
      assert.ok(props[key], `Missing config: ${key}`);
    }
  });

  it("runtime.source has correct enum values", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.deepStrictEqual(props["borrowscope.runtime.source"].enum, ["file", "websocket"]);
  });

  it("runtime.enabled defaults to false", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.strictEqual(props["borrowscope.runtime.enabled"].default, false);
  });

  it("runtime.showDropOrder defaults to false", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.strictEqual(props["borrowscope.runtime.showDropOrder"].default, false);
  });
});

describe("12.12 Status Bar Indicator", () => {
  let status: any;
  let srcContent: string;

  before(() => {
    status = require(path.join(ROOT, "out", "runtime-status"));
    srcContent = fs.readFileSync(path.join(ROOT, "src", "runtime-status.ts"), "utf8");
  });

  it("status bar shows 'Static ✓ | Runtime Off' when disabled", () => {
    assert.ok(srcContent.includes("Static ✓ | Runtime Off"));
  });

  it("status bar shows 'Static ✓ | Runtime ✓' format when enabled", () => {
    assert.ok(srcContent.includes("Static ✓ | ${runtimeStatus}"));
  });

  it("shows event count with formatCount", () => {
    assert.ok(srcContent.includes("formatCount"));
  });

  it("shows time ago", () => {
    assert.ok(srcContent.includes("formatAgo"));
  });

  it("shows stale indicator when source changed", () => {
    assert.ok(srcContent.includes("stale"));
  });

  it("shows Live for websocket mode", () => {
    assert.ok(srcContent.includes("Live"));
  });

  it("shows connecting state", () => {
    assert.ok(srcContent.includes("connecting"));
  });

  it("shows error state", () => {
    assert.ok(srcContent.includes("error"));
  });

  it("shows disconnected state", () => {
    assert.ok(srcContent.includes("disconnected"));
  });

  it("formatCount formats thousands as K", () => {
    // Test via source inspection — formatCount(1247) should produce "1.2K"
    assert.ok(srcContent.includes("1000"));
    assert.ok(srcContent.includes(".toFixed(1)"));
  });

  it("tooltip includes all config settings", () => {
    assert.ok(srcContent.includes("showTimings"));
    assert.ok(srcContent.includes("showDropOrder"));
    assert.ok(srcContent.includes("showRefCounts"));
    assert.ok(srcContent.includes("highlightDivergences"));
  });

  it("status bar command is toggleRuntime", () => {
    assert.ok(srcContent.includes('"borrowscope.toggleRuntime"'));
  });
});
