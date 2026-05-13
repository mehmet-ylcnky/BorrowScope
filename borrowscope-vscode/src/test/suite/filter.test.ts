import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("5.8 Filter by Category", () => {
  let panelSrc: string;

  before(() => {
    panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
  });

  // 1. Filter bar exists in HTML
  it("HTML contains filter-bar div", () => {
    assert.ok(panelSrc.includes('id="filter-bar"'));
  });

  // 2. Filter buttons created per category
  it("creates filter buttons from node categories", () => {
    assert.ok(panelSrc.includes("categories.forEach"));
    assert.ok(panelSrc.includes("append('button')"));
  });

  // 3. Buttons colored by category
  it("filter buttons colored by nodeColor", () => {
    assert.ok(panelSrc.includes("style('background', nodeColor(cat))"));
  });

  // 4. Clicking toggles hidden state
  it("clicking button toggles hiddenCategories set", () => {
    assert.ok(panelSrc.includes("hiddenCategories.has(cat)"));
    assert.ok(panelSrc.includes("hiddenCategories.delete(cat)"));
    assert.ok(panelSrc.includes("hiddenCategories.add(cat)"));
  });

  // 5. Hidden button gets visual indicator
  it("hidden button gets strikethrough class", () => {
    assert.ok(panelSrc.includes("classed('hidden'"));
    assert.ok(panelSrc.includes("text-decoration:line-through"));
  });

  // 6. applyFilters hides nodes
  it("applyFilters hides nodes of hidden categories", () => {
    assert.ok(panelSrc.includes("node.attr('display'"));
    assert.ok(panelSrc.includes("'none'"));
  });

  // 7. applyFilters hides connected edges
  it("applyFilters hides edges connected to hidden nodes", () => {
    assert.ok(panelSrc.includes("edge.attr('display'"));
    assert.ok(panelSrc.includes("srcHidden || tgtHidden"));
  });

  // 8. Filter bar has label
  it("filter bar has Filter label", () => {
    assert.ok(panelSrc.includes("Filter:"));
  });

  // 9. Hidden button has reduced opacity
  it("hidden button has reduced opacity in CSS", () => {
    assert.ok(panelSrc.includes("button.hidden { opacity:0.3"));
  });

  // 10. Categories derived from actual graph data
  it("categories extracted from data.nodes", () => {
    assert.ok(panelSrc.includes("new Set(data.nodes.map(n => n.category))"));
  });
});
