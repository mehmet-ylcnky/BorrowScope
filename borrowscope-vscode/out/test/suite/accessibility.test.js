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
describe("8.7 Accessibility", () => {
    let pkgJson;
    let panelSrc;
    let extSrc;
    before(() => {
        pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
        panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
        extSrc = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
    });
    // === ARIA attributes on graph nodes ===
    it("graph nodes have role='button'", () => {
        assert.ok(panelSrc.includes("'role', 'button'"));
    });
    it("graph nodes have tabindex='0'", () => {
        assert.ok(panelSrc.includes("'tabindex', '0'"));
    });
    it("graph nodes have aria-label with name, type, category", () => {
        assert.ok(panelSrc.includes("aria-label"));
        assert.ok(panelSrc.includes("d.name"));
        assert.ok(panelSrc.includes("d.type"));
        assert.ok(panelSrc.includes("d.category"));
    });
    // === Keyboard navigation ===
    it("Enter key on node triggers navigation", () => {
        assert.ok(panelSrc.includes("event.key === 'Enter'"));
        assert.ok(panelSrc.includes("nodeClicked"));
    });
    // === Screen reader description ===
    it("graph-description aria-live region exists", () => {
        assert.ok(panelSrc.includes('id="graph-description"'));
        assert.ok(panelSrc.includes('aria-live="polite"'));
    });
    it("graph description includes function name", () => {
        assert.ok(panelSrc.includes("rawGraph.function_name"));
    });
    it("graph description includes variable count", () => {
        assert.ok(panelSrc.includes("variables.length"));
    });
    it("graph description lists borrows", () => {
        assert.ok(panelSrc.includes("borrows "));
    });
    it("graph-container has role='img' and aria-label", () => {
        assert.ok(panelSrc.includes('role="img"'));
        assert.ok(panelSrc.includes('aria-label="Ownership graph visualization"'));
    });
    // === Describe Graph command ===
    it("describeGraph command in package.json", () => {
        assert.ok(pkgJson.contributes.commands.some((c) => c.command === "borrowscope.describeGraph"));
    });
    it("describeGraph function exists in extension.ts", () => {
        assert.ok(extSrc.includes("function describeGraph"));
    });
    it("describeGraph shows modal with text description", () => {
        assert.ok(extSrc.includes("Graph Description"));
        assert.ok(extSrc.includes("modal: true"));
    });
    it("describeGraph lists variables with types", () => {
        assert.ok(extSrc.includes("v.type_display"));
        assert.ok(extSrc.includes("v.ownership_category"));
    });
    // === High contrast support ===
    it("panel has high contrast CSS rules", () => {
        assert.ok(panelSrc.includes(".vscode-high-contrast"));
        assert.ok(panelSrc.includes("stroke-width:3px"));
    });
});
//# sourceMappingURL=accessibility.test.js.map