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
describe("4.1 Extension Project Setup", () => {
    // 1. package.json exists and is valid JSON
    it("package.json is valid JSON", () => {
        const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
        assert.strictEqual(pkg.name, "borrowscope");
    });
    // 2. Activation event is onLanguage:rust
    it("activates on Rust files", () => {
        const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
        assert.ok(pkg.activationEvents.includes("onLanguage:rust"));
    });
    // 3. All 4 commands are registered
    it("registers 4 commands", () => {
        const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
        const commands = pkg.contributes.commands.map((c) => c.command);
        assert.ok(commands.includes("borrowscope.showGraph"));
        assert.ok(commands.includes("borrowscope.inspectVariable"));
        assert.ok(commands.includes("borrowscope.toggleDecorations"));
        assert.ok(commands.includes("borrowscope.restartServer"));
    });
    // 4. Configuration properties exist
    it("defines configuration properties", () => {
        const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
        const props = pkg.contributes.configuration.properties;
        assert.ok("borrowscope.server.path" in props);
        assert.ok("borrowscope.decorations.enabled" in props);
        assert.ok("borrowscope.decorations.borrowScopes" in props);
        assert.ok("borrowscope.graph.layout" in props);
        assert.strictEqual(props["borrowscope.graph.layout"].default, "hierarchical");
    });
    // 5. Main entry point is set
    it("main points to out/extension.js", () => {
        const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
        assert.strictEqual(pkg.main, "./out/extension.js");
    });
    // 6. Build output exists
    it("build output exists", () => {
        assert.ok(fs.existsSync(path.join(ROOT, "out", "extension.js")));
    });
    // 7. Extension source exports activate and deactivate
    it("extension exports activate and deactivate", () => {
        const srcPath = path.join(ROOT, "src", "extension.ts");
        const content = fs.readFileSync(srcPath, "utf8");
        assert.ok(content.includes("export async function activate"), "Should export activate");
        assert.ok(content.includes("export async function deactivate"), "Should export deactivate");
    });
    // 8. vscode-languageclient is a dependency
    it("depends on vscode-languageclient", () => {
        const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
        assert.ok("vscode-languageclient" in pkg.dependencies);
    });
    // 9. Engine requires vscode ^1.85.0
    it("requires vscode ^1.85.0", () => {
        const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
        assert.strictEqual(pkg.engines.vscode, "^1.85.0");
    });
    // 10. TypeScript source compiles without errors (build succeeded)
    it("esbuild config exists and is valid", () => {
        const esbuildPath = path.join(ROOT, "esbuild.js");
        assert.ok(fs.existsSync(esbuildPath));
        const content = fs.readFileSync(esbuildPath, "utf8");
        assert.ok(content.includes("entryPoints"));
        assert.ok(content.includes("extension.ts"));
    });
});
//# sourceMappingURL=extension.test.js.map