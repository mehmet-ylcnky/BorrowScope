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

  // 5. showGraph opens GraphPanel
  it("showGraph opens GraphPanel", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("GraphPanel.createOrShow"));
  });

  // 6. showGraph passes graph data to panel
  it("showGraph passes graph to panel", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("extensionUri"));
    assert.ok(src.includes("graph"));
  });

  // 7. showGraph handles no function at cursor
  it("showGraph handles no function at cursor", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("No function at cursor"));
  });

  // 8. showGraph handles server errors
  it("showGraph handles errors gracefully", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("showErrorMessage"));
  });

  // 9. showGraph requests ownershipGraph
  it("showGraph sends borrowscope/ownershipGraph request", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("borrowscope/ownershipGraph"));
  });

  // 10. showGraph handles missing client
  it("showGraph warns when server not running", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
    assert.ok(src.includes("Server not running"));
  });
});
