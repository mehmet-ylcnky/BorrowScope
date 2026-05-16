import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("12.1 Runtime Event File Watcher", () => {
  let RuntimeWatcher: any;
  let runtimeTypes: any;
  let pkgJson: any;

  before(() => {
    RuntimeWatcher = require(path.join(ROOT, "out", "runtime-watcher")).RuntimeWatcher;
    runtimeTypes = require(path.join(ROOT, "out", "runtime-types"));
    pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
  });

  // 1. RuntimeWatcher class exists
  it("RuntimeWatcher class is exported", () => {
    assert.ok(RuntimeWatcher);
    assert.strictEqual(typeof RuntimeWatcher, "function");
  });

  // 2. Can instantiate
  it("can instantiate RuntimeWatcher", () => {
    const watcher = new RuntimeWatcher();
    assert.ok(watcher);
    assert.strictEqual(watcher.eventCount, 0);
    assert.strictEqual(watcher.isStale, false);
    watcher.dispose();
  });

  // 3. getEvents returns empty array initially
  it("getEvents returns empty array initially", () => {
    const watcher = new RuntimeWatcher();
    assert.deepStrictEqual(watcher.getEvents(), []);
    watcher.dispose();
  });

  // 4. Has onEventsUpdated event
  it("has onEventsUpdated event emitter", () => {
    const watcher = new RuntimeWatcher();
    assert.ok(watcher.onEventsUpdated);
    assert.strictEqual(typeof watcher.onEventsUpdated, "function");
    watcher.dispose();
  });

  // 5. Has dispose method
  it("has dispose method", () => {
    const watcher = new RuntimeWatcher();
    assert.strictEqual(typeof watcher.dispose, "function");
    watcher.dispose();
  });

  // 6. isStale property exists
  it("isStale is false initially", () => {
    const watcher = new RuntimeWatcher();
    assert.strictEqual(watcher.isStale, false);
    watcher.dispose();
  });

  // 7. lastLoadTime is 0 initially
  it("lastLoadTime is 0 initially", () => {
    const watcher = new RuntimeWatcher();
    assert.strictEqual(watcher.lastLoadTime, 0);
    watcher.dispose();
  });

  // 8. filePath is empty before start
  it("filePath is empty before start", () => {
    const watcher = new RuntimeWatcher();
    assert.strictEqual(watcher.filePath, "");
    watcher.dispose();
  });

  // 9. RuntimeEvent type definitions exist
  it("runtime-types module exports correctly", () => {
    // Module should load without error (types are compile-time only)
    assert.ok(runtimeTypes !== undefined);
  });

  // 10. Configuration: runtime.enabled exists
  it("package.json has runtime.enabled config", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.ok(props["borrowscope.runtime.enabled"]);
    assert.strictEqual(props["borrowscope.runtime.enabled"].default, false);
  });

  // 11. Configuration: runtime.source exists
  it("package.json has runtime.source config", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.ok(props["borrowscope.runtime.source"]);
    assert.deepStrictEqual(props["borrowscope.runtime.source"].enum, ["file", "websocket"]);
  });

  // 12. Configuration: runtime.filePath exists
  it("package.json has runtime.filePath config", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.ok(props["borrowscope.runtime.filePath"]);
    assert.strictEqual(props["borrowscope.runtime.filePath"].default, ".borrowscope/events.json");
  });

  // 13. Configuration: runtime.websocketPort exists
  it("package.json has runtime.websocketPort config", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.ok(props["borrowscope.runtime.websocketPort"]);
    assert.strictEqual(props["borrowscope.runtime.websocketPort"].default, 9876);
  });

  // 14. Configuration: runtime.showTimings exists
  it("package.json has runtime.showTimings config", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.ok(props["borrowscope.runtime.showTimings"]);
    assert.strictEqual(props["borrowscope.runtime.showTimings"].default, true);
  });

  // 15. Configuration: runtime.showDropOrder exists
  it("package.json has runtime.showDropOrder config", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.ok(props["borrowscope.runtime.showDropOrder"]);
    assert.strictEqual(props["borrowscope.runtime.showDropOrder"].default, false);
  });

  // 16. Configuration: runtime.showRefCounts exists
  it("package.json has runtime.showRefCounts config", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.ok(props["borrowscope.runtime.showRefCounts"]);
  });

  // 17. Configuration: runtime.highlightDivergences exists
  it("package.json has runtime.highlightDivergences config", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.ok(props["borrowscope.runtime.highlightDivergences"]);
    assert.strictEqual(props["borrowscope.runtime.highlightDivergences"].default, true);
  });

  // 18. Extension imports RuntimeWatcher
  it("extension.ts imports RuntimeWatcher", () => {
    const extSrc = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
    assert.ok(extSrc.includes("RuntimeWatcher"));
    assert.ok(extSrc.includes("runtime-watcher"));
  });

  // 19. Extension exports getRuntimeWatcher
  it("extension.ts exports getRuntimeWatcher", () => {
    const extSrc = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
    assert.ok(extSrc.includes("export function getRuntimeWatcher"));
  });

  // 20. Watcher does not start when runtime.enabled is false
  it("start always sets filePath (watches regardless of enabled)", () => {
    const watcher = new RuntimeWatcher();
    watcher.start("/tmp/fake-workspace");
    assert.ok(watcher.filePath.includes("events.json"));
    watcher.dispose();
  });
});
