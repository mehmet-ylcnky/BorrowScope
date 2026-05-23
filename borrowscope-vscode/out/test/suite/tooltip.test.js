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
describe("5.7 Hover for Type Details", () => {
    let panelSrc;
    before(() => {
        panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
    });
    // 1. Tooltip div exists in HTML
    it("HTML contains tooltip div", () => {
        assert.ok(panelSrc.includes('id="tooltip"'));
    });
    // 2. Tooltip shows variable name
    it("tooltip shows variable name on hover", () => {
        assert.ok(panelSrc.includes("d.name"));
        assert.ok(panelSrc.includes("<strong>"));
    });
    // 3. Tooltip shows type
    it("tooltip shows type", () => {
        assert.ok(panelSrc.includes("d.type"));
        assert.ok(panelSrc.includes("<code>"));
    });
    // 4. Tooltip shows category
    it("tooltip shows ownership category", () => {
        assert.ok(panelSrc.includes("d.category"));
        assert.ok(panelSrc.includes("Category:"));
    });
    // 5. Tooltip shows line number
    it("tooltip shows line number", () => {
        assert.ok(panelSrc.includes("d.line"));
        assert.ok(panelSrc.includes("Line:"));
    });
    // 6. Tooltip shows copy status
    it("tooltip shows copy type indicator", () => {
        assert.ok(panelSrc.includes("d.isCopy"));
        assert.ok(panelSrc.includes("Copy type"));
    });
    // 7. Tooltip shows moved/dropped status
    it("tooltip shows moved/dropped status", () => {
        assert.ok(panelSrc.includes("d.isAlive"));
        assert.ok(panelSrc.includes("Moved / dropped"));
    });
    // 8. Tooltip positioned near cursor
    it("tooltip positioned at mouse coordinates", () => {
        assert.ok(panelSrc.includes("event.pageX"));
        assert.ok(panelSrc.includes("event.pageY"));
    });
    // 9. Tooltip hidden on mouseout
    it("tooltip hidden on mouseout", () => {
        assert.ok(panelSrc.includes("opacity', 0"));
    });
    // 10. Tooltip has proper styling
    it("tooltip has background and border styling", () => {
        assert.ok(panelSrc.includes("editorHoverWidget-background"));
        assert.ok(panelSrc.includes("editorHoverWidget-border"));
        assert.ok(panelSrc.includes("border-radius"));
    });
});
//# sourceMappingURL=tooltip.test.js.map