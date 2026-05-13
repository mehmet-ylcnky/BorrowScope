import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("5.5 Edge Styling", () => {
  // These tests verify the D3 rendering code has correct edge styles
  // by checking the panel source code for the style definitions

  let panelSrc: string;

  before(() => {
    panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
  });

  // 1. Shared borrow edge is blue
  it("shared borrow edge color is blue", () => {
    assert.ok(panelSrc.includes("shared_borrow: '#3498db'"));
  });

  // 2. Shared borrow edge is dashed
  it("shared borrow edge is dashed", () => {
    assert.ok(panelSrc.includes("shared_borrow") && panelSrc.includes("'8,4'"));
  });

  // 3. Mutable borrow edge is red
  it("mutable borrow edge color is red", () => {
    assert.ok(panelSrc.includes("mutable_borrow: '#e74c3c'"));
  });

  // 4. Mutable borrow edge is solid thick
  it("mutable borrow edge is solid (no dash)", () => {
    // mutable_borrow returns 'none' for dash
    assert.ok(panelSrc.includes("'mutable_borrow') return 'none'"));
  });

  // 5. Move edge is green
  it("move edge color is green", () => {
    assert.ok(panelSrc.includes("move: '#2ecc71'"));
  });

  // 6. Move edge is solid
  it("move edge is solid (no dash)", () => {
    assert.ok(panelSrc.includes("'move') return 'none'"));
  });

  // 7. Rc clone edge is purple
  it("Rc clone edge color is purple", () => {
    assert.ok(panelSrc.includes("rc_clone: '#9b59b6'"));
  });

  // 8. Clone edge is dotted
  it("clone edge is dotted", () => {
    assert.ok(panelSrc.includes("rc_clone") && panelSrc.includes("'3,3'"));
  });

  // 9. Closure capture edge is orange dash-dot
  it("closure capture edge is orange", () => {
    assert.ok(panelSrc.includes("closure_capture: '#e67e22'"));
  });

  // 10. Closure capture has dash-dot pattern
  it("closure capture has dash-dot pattern", () => {
    assert.ok(panelSrc.includes("closure_capture") && panelSrc.includes("'8,3,2,3'"));
  });

  // 11. Arrow markers defined for all edge types
  it("arrow markers defined for all edge types", () => {
    assert.ok(panelSrc.includes("shared_borrow"));
    assert.ok(panelSrc.includes("mutable_borrow"));
    assert.ok(panelSrc.includes("move"));
    assert.ok(panelSrc.includes("rc_clone"));
    assert.ok(panelSrc.includes("arc_clone"));
    assert.ok(panelSrc.includes("closure_capture"));
  });

  // 12. Mutable borrow is thicker than shared
  it("mutable borrow stroke is thicker", () => {
    assert.ok(panelSrc.includes("d.isMutable ? 2.5 : 1.5"));
  });

  // 13. Edge labels are rendered
  it("edge labels are rendered as text", () => {
    assert.ok(panelSrc.includes("edgeLabel"));
    assert.ok(panelSrc.includes("text-anchor', 'middle'"));
  });

  // 14. Arrows point from source to target
  it("arrows use marker-end (source to target direction)", () => {
    assert.ok(panelSrc.includes("marker-end"));
  });

  // 15. Arc clone has distinct color from Rc
  it("Arc clone has different color than Rc clone", () => {
    assert.ok(panelSrc.includes("arc_clone: '#8e44ad'"));
    // 8e44ad != 9b59b6
  });
});
