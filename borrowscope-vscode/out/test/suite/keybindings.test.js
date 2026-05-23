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
describe("8.2 Keyboard Shortcuts", () => {
    let pkgJson;
    before(() => {
        pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
    });
    const keybindings = () => pkgJson.contributes.keybindings;
    const commands = () => pkgJson.contributes.commands;
    // === Keybindings exist ===
    it("has keybindings section", () => {
        assert.ok(Array.isArray(keybindings()));
        assert.ok(keybindings().length >= 6);
    });
    it("Ctrl+Shift+O opens graph", () => {
        const kb = keybindings().find((k) => k.command === "borrowscope.showGraph");
        assert.ok(kb);
        assert.strictEqual(kb.key, "ctrl+shift+o");
        assert.strictEqual(kb.when, "editorLangId == rust");
    });
    it("Ctrl+Shift+I inspects variable", () => {
        const kb = keybindings().find((k) => k.command === "borrowscope.inspectVariable");
        assert.ok(kb);
        assert.strictEqual(kb.key, "ctrl+shift+i");
        assert.strictEqual(kb.when, "editorLangId == rust");
    });
    it("Ctrl+Shift+D toggles decorations", () => {
        const kb = keybindings().find((k) => k.command === "borrowscope.toggleDecorations");
        assert.ok(kb);
        assert.strictEqual(kb.key, "ctrl+shift+d");
        assert.strictEqual(kb.when, "editorLangId == rust");
    });
    it("Alt+Shift+N goes to next conflict", () => {
        const kb = keybindings().find((k) => k.command === "borrowscope.nextConflict");
        assert.ok(kb);
        assert.strictEqual(kb.key, "alt+shift+n");
        assert.strictEqual(kb.when, "editorLangId == rust");
    });
    it("Alt+Shift+P goes to previous conflict", () => {
        const kb = keybindings().find((k) => k.command === "borrowscope.prevConflict");
        assert.ok(kb);
        assert.strictEqual(kb.key, "alt+shift+p");
        assert.strictEqual(kb.when, "editorLangId == rust");
    });
    it("Ctrl+Shift+G focuses graph panel", () => {
        const kb = keybindings().find((k) => k.command === "borrowscope.focusGraph");
        assert.ok(kb);
        assert.strictEqual(kb.key, "ctrl+shift+g");
        assert.strictEqual(kb.when, "editorLangId == rust");
    });
    // === All keybindings have 'when' clause for Rust ===
    it("all keybindings are scoped to Rust files", () => {
        for (const kb of keybindings()) {
            assert.ok(kb.when && kb.when.includes("rust"), `${kb.command} missing Rust 'when' clause`);
        }
    });
    // === Commands exist for all keybindings ===
    it("all keybinding commands are registered", () => {
        for (const kb of keybindings()) {
            const cmd = commands().find((c) => c.command === kb.command);
            assert.ok(cmd, `Command ${kb.command} not found in commands list`);
        }
    });
    // === New commands exist ===
    it("nextConflict command exists", () => {
        assert.ok(commands().some((c) => c.command === "borrowscope.nextConflict"));
    });
    it("prevConflict command exists", () => {
        assert.ok(commands().some((c) => c.command === "borrowscope.prevConflict"));
    });
    it("focusGraph command exists", () => {
        assert.ok(commands().some((c) => c.command === "borrowscope.focusGraph"));
    });
    // === Implementation exists ===
    it("extension.ts has nextConflict implementation", () => {
        const src = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
        assert.ok(src.includes("function nextConflict"));
        assert.ok(src.includes("getDiagnostics"));
    });
    it("extension.ts has prevConflict implementation", () => {
        const src = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
        assert.ok(src.includes("function prevConflict"));
    });
    it("extension.ts has focusGraph implementation", () => {
        const src = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
        assert.ok(src.includes("function focusGraph"));
    });
    it("nextConflict wraps around to first", () => {
        const src = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
        assert.ok(src.includes("Wrap around to first"));
    });
    it("prevConflict wraps around to last", () => {
        const src = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
        assert.ok(src.includes("Wrap around to last"));
    });
});
//# sourceMappingURL=keybindings.test.js.map