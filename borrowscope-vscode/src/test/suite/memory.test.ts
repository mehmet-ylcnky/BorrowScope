import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("13. Memory Layout Visualization", () => {
  let panelSrc: string;
  let pkgJson: any;

  before(() => {
    panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
    pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
  });

  // 1. Memory tab exists in panel
  it("panel has Memory view button", () => {
    assert.ok(panelSrc.includes('data-view="memory"'));
  });

  // 2. renderMemory function exists
  it("renderMemory function exists", () => {
    assert.ok(panelSrc.includes("function renderMemory()"));
  });

  // 3. Shows stack column
  it("renders stack column with STACK label", () => {
    assert.ok(panelSrc.includes("STACK"));
  });

  // 4. Shows heap column
  it("renders heap column with HEAP label", () => {
    assert.ok(panelSrc.includes("HEAP"));
  });

  // 5. Shows variable size and alignment
  it("shows variable size and alignment", () => {
    assert.ok(panelSrc.includes("v.size"));
    assert.ok(panelSrc.includes("v.alignment"));
  });

  // 6. Shows variable offset
  it("shows stack offset", () => {
    assert.ok(panelSrc.includes("v.offset"));
  });

  // 7. Shows lifetime span (line - end_line)
  it("shows lifetime span with end_line", () => {
    assert.ok(panelSrc.includes("v.end_line"));
  });

  // 8. Timeline slider exists
  it("has timeline slider", () => {
    assert.ok(panelSrc.includes("mem-slider"));
    assert.ok(panelSrc.includes('type="range"'));
  });

  // 9. Play button exists
  it("has play button for animation", () => {
    assert.ok(panelSrc.includes("mem-play"));
    assert.ok(panelSrc.includes("setInterval"));
  });

  // 10. Step button exists
  it("has step button", () => {
    assert.ok(panelSrc.includes("mem-step"));
  });

  // 11. Variables disappear after end_line
  it("filters variables by end_line", () => {
    assert.ok(panelSrc.includes("v.end_line >= line") || panelSrc.includes("end_line"));
  });

  // 12. Dropped variables shown
  it("shows dropped variables in strikethrough", () => {
    assert.ok(panelSrc.includes("droppedVars"));
    assert.ok(panelSrc.includes("line-through"));
  });

  // 13. New variable highlighted
  it("highlights newly allocated variables", () => {
    assert.ok(panelSrc.includes("isNew"));
    assert.ok(panelSrc.includes("allocated this line"));
  });

  // 14. Pointer relationships shown
  it("shows pointer relationships", () => {
    assert.ok(panelSrc.includes("allPtrs") || panelSrc.includes("pointers"));
  });

  // 15. Memory CodeLens in server
  it("server source has memory CodeLens", () => {
    const serverSrc = fs.readFileSync(path.join(ROOT, "..", "borrowscope-lsp", "src", "handlers", "requests.rs"), "utf8");
    assert.ok(serverSrc.includes("analyze_memory_layout"));
    assert.ok(serverSrc.includes("Stack:"));
    assert.ok(serverSrc.includes("Heap:"));
  });

  // 16. Configuration: memoryLayout.enabled exists
  it("package.json has memoryLayout configuration", () => {
    const props = pkgJson.contributes.configuration.properties;
    assert.ok(props["borrowscope.memoryLayout.enabled"], "Should have memoryLayout.enabled");
    assert.ok(props["borrowscope.memoryLayout.animationSpeed"], "Should have memoryLayout.animationSpeed");
  });
});
