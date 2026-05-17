import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("8.4 Theme Integration", () => {
  let pkgJson: any;
  let panelSrc: string;
  let decSrc: string;

  before(() => {
    pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
    panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
    decSrc = fs.readFileSync(path.join(ROOT, "src", "decorations.ts"), "utf8");
  });

  // === Theme colors registered ===
  it("package.json has colors contribution", () => {
    assert.ok(pkgJson.contributes.colors);
    assert.ok(Array.isArray(pkgJson.contributes.colors));
  });

  it("has sharedBorrowBackground color", () => {
    const c = pkgJson.contributes.colors.find((c: any) => c.id === "borrowscope.sharedBorrowBackground");
    assert.ok(c);
    assert.ok(c.defaults.dark);
    assert.ok(c.defaults.light);
    assert.ok(c.defaults.highContrast);
  });

  it("has sharedBorrowBorder color", () => {
    const c = pkgJson.contributes.colors.find((c: any) => c.id === "borrowscope.sharedBorrowBorder");
    assert.ok(c);
  });

  it("has mutBorrowBackground color", () => {
    assert.ok(pkgJson.contributes.colors.find((c: any) => c.id === "borrowscope.mutBorrowBackground"));
  });

  it("has mutBorrowBorder color", () => {
    assert.ok(pkgJson.contributes.colors.find((c: any) => c.id === "borrowscope.mutBorrowBorder"));
  });

  it("has moveHighlight color", () => {
    assert.ok(pkgJson.contributes.colors.find((c: any) => c.id === "borrowscope.moveHighlight"));
  });

  it("has rcArcHighlight color", () => {
    assert.ok(pkgJson.contributes.colors.find((c: any) => c.id === "borrowscope.rcArcHighlight"));
  });

  it("has conflictBackground color", () => {
    assert.ok(pkgJson.contributes.colors.find((c: any) => c.id === "borrowscope.conflictBackground"));
  });

  it("has dropIndicator color", () => {
    assert.ok(pkgJson.contributes.colors.find((c: any) => c.id === "borrowscope.dropIndicator"));
  });

  it("has runtimeTiming color", () => {
    assert.ok(pkgJson.contributes.colors.find((c: any) => c.id === "borrowscope.runtimeTiming"));
  });

  it("has divergenceHighlight color", () => {
    assert.ok(pkgJson.contributes.colors.find((c: any) => c.id === "borrowscope.divergenceHighlight"));
  });

  it("has 10 theme colors total", () => {
    assert.strictEqual(pkgJson.contributes.colors.length, 10);
  });

  // === All theme colors have dark/light/highContrast defaults ===
  it("all colors have dark, light, and highContrast defaults", () => {
    for (const c of pkgJson.contributes.colors) {
      assert.ok(c.defaults.dark, `${c.id} missing dark default`);
      assert.ok(c.defaults.light, `${c.id} missing light default`);
      assert.ok(c.defaults.highContrast, `${c.id} missing highContrast default`);
    }
  });

  // === Panel uses VS Code CSS variables ===
  it("panel uses --vscode-editor-background", () => {
    assert.ok(panelSrc.includes("--vscode-editor-background"));
  });

  it("panel uses --vscode-editor-foreground", () => {
    assert.ok(panelSrc.includes("--vscode-editor-foreground"));
  });

  it("panel uses --vscode-panel-border", () => {
    assert.ok(panelSrc.includes("--vscode-panel-border"));
  });

  it("panel has light theme CSS rules", () => {
    assert.ok(panelSrc.includes(".vscode-light"));
  });

  it("panel has high contrast CSS rules", () => {
    assert.ok(panelSrc.includes(".vscode-high-contrast"));
  });

  it("high contrast uses thicker borders", () => {
    assert.ok(panelSrc.includes("stroke-width:3px"));
  });

  // === Decorations use config colors ===
  it("decorations read colors from config", () => {
    assert.ok(decSrc.includes("getConfiguration"));
    assert.ok(decSrc.includes("borrowscope.colors"));
  });

  it("getColorForLabel uses config for shared borrow", () => {
    assert.ok(decSrc.includes('cfg.get("sharedBorrow"'));
  });

  it("getColorForLabel uses config for mutable borrow", () => {
    assert.ok(decSrc.includes('cfg.get("mutableBorrow"'));
  });

  it("getColorForLabel uses config for Rc/Arc", () => {
    assert.ok(decSrc.includes('cfg.get("rcArc"'));
  });
});
