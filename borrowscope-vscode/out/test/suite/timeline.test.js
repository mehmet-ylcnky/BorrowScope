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
describe("7.1 Temporal View: Timeline", () => {
    let timeline;
    let panelSrc;
    before(() => {
        timeline = require(path.join(ROOT, "out", "graph", "timeline.js"));
        panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
    });
    // 1. buildTimelineData returns correct structure
    it("buildTimelineData returns variables and borrowScopes", () => {
        const graph = { function_name: "test", end_line: 10, variables: [{ name: "x", line: 2, type_display: "i32", ownership_category: "Copy" }], borrow_scopes: [], conflicts: [] };
        const result = timeline.buildTimelineData(graph);
        assert.strictEqual(result.functionName, "test");
        assert.strictEqual(result.variables.length, 1);
        assert.strictEqual(result.variables[0].name, "x");
    });
    // 2. Variable startLine and endLine set correctly
    it("variable has correct start and end lines", () => {
        const graph = { function_name: "f", end_line: 15, variables: [{ name: "a", line: 3, type_display: "Vec<i32>", ownership_category: "Owned" }], borrow_scopes: [], conflicts: [] };
        const result = timeline.buildTimelineData(graph);
        assert.strictEqual(result.variables[0].startLine, 3);
        assert.ok(result.variables[0].endLine >= 3);
    });
    // 3. Borrow scopes mapped correctly
    it("borrow scopes have borrower, target, mutable, lines", () => {
        const graph = { function_name: "f", end_line: 10, variables: [{ name: "data", line: 2, type_display: "Vec", ownership_category: "Owned" }, { name: "r", line: 3, type_display: "&Vec", ownership_category: "SharedRef" }], borrow_scopes: [{ borrower_name: "r", target_name: "data", is_mutable: false, start_line: 3, end_line: 7 }], conflicts: [] };
        const result = timeline.buildTimelineData(graph);
        assert.strictEqual(result.borrowScopes.length, 1);
        assert.strictEqual(result.borrowScopes[0].borrower, "r");
        assert.strictEqual(result.borrowScopes[0].target, "data");
        assert.strictEqual(result.borrowScopes[0].isMutable, false);
        assert.strictEqual(result.borrowScopes[0].startLine, 3);
        assert.strictEqual(result.borrowScopes[0].endLine, 7);
    });
    // 4. Conflicts mapped correctly
    it("conflicts have start and end lines", () => {
        const graph = { function_name: "f", end_line: 10, variables: [], borrow_scopes: [], conflicts: [{ variable: "data", borrow_a: "r", borrow_b: "m", overlap_start_line: 5, overlap_end_line: 8 }] };
        const result = timeline.buildTimelineData(graph);
        assert.strictEqual(result.conflicts.length, 1);
        assert.strictEqual(result.conflicts[0].startLine, 5);
        assert.strictEqual(result.conflicts[0].endLine, 8);
    });
    // 5. minLine and maxLine computed from variables
    it("computes minLine and maxLine", () => {
        const graph = { function_name: "f", end_line: 20, variables: [{ name: "a", line: 5, type_display: "i32", ownership_category: "Copy" }, { name: "b", line: 12, type_display: "i32", ownership_category: "Copy" }], borrow_scopes: [], conflicts: [] };
        const result = timeline.buildTimelineData(graph);
        assert.strictEqual(result.minLine, 5);
        assert.ok(result.maxLine >= 12);
    });
    // 6. Empty graph produces empty timeline
    it("empty graph produces empty timeline", () => {
        const graph = { function_name: "empty", end_line: 5, variables: [], borrow_scopes: [], conflicts: [] };
        const result = timeline.buildTimelineData(graph);
        assert.strictEqual(result.variables.length, 0);
        assert.strictEqual(result.borrowScopes.length, 0);
    });
    // 7. Panel HTML contains timeline-container
    it("panel HTML has timeline container", () => {
        assert.ok(panelSrc.includes('id="timeline-container"'));
    });
    // 8. Panel has view toggle buttons
    it("panel has Graph and Timeline view buttons", () => {
        assert.ok(panelSrc.includes('data-view="graph"'));
        assert.ok(panelSrc.includes('data-view="timeline"'));
    });
    // 9. Timeline renders bars with D3
    it("timeline script renders variable bars", () => {
        assert.ok(panelSrc.includes("renderTimeline"));
        assert.ok(panelSrc.includes("barH"));
        assert.ok(panelSrc.includes("scaleLinear"));
    });
    // 10. Timeline shows borrow overlays
    it("timeline renders borrow overlays on target bars", () => {
        assert.ok(panelSrc.includes("s.target === v.name"));
        assert.ok(panelSrc.includes("s.mutable"));
    });
    // 11. Timeline shows conflict zones
    it("timeline renders conflict zones as red bands", () => {
        assert.ok(panelSrc.includes("conflicts"));
        assert.ok(panelSrc.includes("rgba(231,76,60"));
    });
    // 12. Clicking a bar sends nodeClicked message
    it("clicking timeline bar navigates to source", () => {
        assert.ok(panelSrc.includes("nodeClicked"));
        assert.ok(panelSrc.includes("v.start"));
    });
});
//# sourceMappingURL=timeline.test.js.map