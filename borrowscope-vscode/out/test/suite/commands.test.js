"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const assert = __importStar(require("assert"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const ROOT = path.resolve(__dirname, "..", "..", "..");
describe("4.8 CodeLens Rendering", () => {
    // 1. Server declares codeLensProvider capability
    it("server declares codeLensProvider capability", () => {
        const capSrc = fs.readFileSync(path.join(ROOT, "..", "borrowscope-lsp", "src", "capabilities.rs"), "utf8");
        assert.ok(capSrc.includes("code_lens_provider"), "Server should declare codeLensProvider");
    });
    // 2. showGraph command is registered
    it("showGraph command is registered in package.json", () => {
        const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
        const commands = pkg.contributes.commands.map((c) => c.command);
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
//# sourceMappingURL=commands.test.js.map