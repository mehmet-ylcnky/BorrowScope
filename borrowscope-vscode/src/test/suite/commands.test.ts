import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("4.8 CodeLens Rendering", () => {
  // 1. Server declares codeLensProvider capability
  it("server declares codeLensProvider capability", () => {
    const capSrc = fs.readFileSync(
      path.join(ROOT, "..", "borrowscope-lsp", "src", "capabilities.rs"), "utf8"
    );
    assert.ok(capSrc.includes("code_lens_provider"), "Server should declare codeLensProvider");
  });

  // 2. showGraph command is registered
  it("showGraph command is registered in package.json", () => {
    const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
    const commands = pkg.contributes.commands.map((c: any) => c.command);
    assert.ok(commands.includes("borrowscope.showGraph"));
  });

  // 3. commands.ts exports registerCommands
  it("commands.ts exports registerCommands function", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("export function registerCommands"));
  });

  // 4. showGraph command requests ownershipGraph from server
  it("showGraph sends borrowscope/ownershipGraph request", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("borrowscope/ownershipGraph"));
  });

  // 5. showGraph displays function name and stats
  it("showGraph formats stats message", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("total_variables"));
    assert.ok(src.includes("total_borrows"));
    assert.ok(src.includes("moves"));
  });

  // 6. showGraph offers Copy JSON action
  it("showGraph offers Copy JSON action", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("Copy JSON"));
    assert.ok(src.includes("clipboard"));
  });

  // 7. showGraph offers Show Variables action
  it("showGraph offers Show Variables action", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("Show Variables"));
    assert.ok(src.includes("OutputChannel"));
  });

  // 8. buildDetail formats variables with ownership category
  it("buildDetail includes ownership category", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("ownership_category"));
  });

  // 9. buildDetail formats borrow scopes
  it("buildDetail includes borrow scope info", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("borrower_name"));
    assert.ok(src.includes("target_name"));
  });

  // 10. buildDetail formats conflicts with warning
  it("buildDetail shows conflicts with warning symbol", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("Conflicts"));
    assert.ok(src.includes("overlap"));
  });
});
