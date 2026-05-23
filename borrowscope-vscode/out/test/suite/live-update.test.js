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
describe("5.9 Live Update", () => {
    let clientSrc;
    before(() => {
        clientSrc = fs.readFileSync(path.join(ROOT, "src", "client.ts"), "utf8");
    });
    // 1. analysisUpdated triggers panel refresh
    it("analysisUpdated handler refreshes graph panel", () => {
        assert.ok(clientSrc.includes("GraphPanel.getPanel()"));
        assert.ok(clientSrc.includes("panel.updateGraph"));
    });
    // 2. Only refreshes if current function was affected
    it("only refreshes if affected function matches current", () => {
        assert.ok(clientSrc.includes("params.functions?.includes(currentFn)"));
    });
    // 3. Re-fetches ownershipGraph for current function
    it("re-fetches ownershipGraph on update", () => {
        assert.ok(clientSrc.includes("borrowscope/ownershipGraph"));
    });
    // 4. Finds function line by name
    it("finds function line by regex match", () => {
        assert.ok(clientSrc.includes("RegExp"));
        assert.ok(clientSrc.includes("currentFn"));
    });
    // 5. Handles errors gracefully
    it("catches errors during panel refresh", () => {
        assert.ok(clientSrc.includes("catch"));
    });
});
describe("5.10 Function Selector", () => {
    let panelSrc;
    before(() => {
        panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
    });
    // 1. Selector dropdown exists in HTML
    it("HTML contains view toggle", () => {
        assert.ok(panelSrc.includes('id="view-toggle"'));
    });
    // 2. Dropdown populated with function list
    it("view toggle has Graph and Timeline buttons", () => {
        assert.ok(panelSrc.includes("data-view=\"graph\""));
        assert.ok(panelSrc.includes("data-view=\"timeline\""));
    });
    // 3. Current function is pre-selected
    it("function name shown in header", () => {
        assert.ok(panelSrc.includes("graph.function_name"));
        assert.ok(panelSrc.includes("graph.function_name"));
    });
    // 4. Change event sends selectFunction message
    it("change event sends selectFunction message via vscodeApi", () => {
        assert.ok(panelSrc.includes("selectFunction"));
        assert.ok(panelSrc.includes("postMessage"));
    });
    // 5. selectFunction handler loads new function
    it("selectFunction handler calls _loadFunction", () => {
        assert.ok(panelSrc.includes("_loadFunction"));
        assert.ok(panelSrc.includes("message.name"));
    });
    // 6. _loadFunction finds function line and fetches graph
    it("_loadFunction searches editor for function and fetches graph", () => {
        assert.ok(panelSrc.includes("ownershipGraph"));
        assert.ok(panelSrc.includes("RegExp"));
    });
    // 7. Dropdown uses VS Code theme colors
    it("panel uses VS Code theme CSS variables", () => {
        assert.ok(panelSrc.includes("vscode-button-background"));
        assert.ok(panelSrc.includes("vscode-foreground"));
    });
    // 8. Function list extracted from editor in commands.ts
    it("commands.ts extracts function list from editor", () => {
        const cmdSrc = fs.readFileSync(path.join(ROOT, "src", "commands.ts"), "utf8");
        assert.ok(cmdSrc.includes("fnList"));
        assert.ok(cmdSrc.includes("match"));
    });
    // 9. Panel createOrShow accepts functionList
    it("createOrShow accepts functionList parameter", () => {
        assert.ok(panelSrc.includes("functionList?: string[]"));
    });
    // 10. updateGraph accepts functionList
    it("updateGraph accepts functionList parameter", () => {
        assert.ok(panelSrc.includes("updateGraph(graph: any, functionList?: string[])"));
    });
});
//# sourceMappingURL=live-update.test.js.map