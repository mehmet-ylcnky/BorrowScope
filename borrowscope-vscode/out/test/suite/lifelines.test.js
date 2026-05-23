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
const ROOT = path.resolve(__dirname, "..", "..", "..");
describe("4.5 Gutter Icons and Lifecycle Flow Lines", () => {
    let lifelines;
    before(() => {
        require("vscode");
        lifelines = require(path.join(ROOT, "out", "lifelines.js"));
    });
    afterEach(() => {
        lifelines.disposeLifelines();
    });
    function mockEditor() {
        const applied = new Map();
        return {
            setDecorations: (dt, ranges) => { applied.set(dt, ranges); },
            __applied: applied,
        };
    }
    const sampleGraph = {
        function_name: "example",
        start_line: 1,
        end_line: 10,
        variables: [
            { name: "data", line: 2, ownership_category: "Owned" },
            { name: "r", line: 3, ownership_category: "SharedRef" },
        ],
        borrow_scopes: [],
        moves: [{ source_name: "data", line: 8, destination: "process", source_type: "Vec<i32>" }],
        rc_clones: [{ clone_variable: "rc2", source_variable: "rc1", clone_type: "Rc", line: 5 }],
        conflicts: [{ variable: "data", borrow_a: "r1", borrow_b: "m1", overlap_start_line: 4, overlap_end_line: 6 }],
    };
    // 1. Owner lifelines disabled (too noisy) — moves shown as single events instead
    it("owner lifelines disabled, moves shown as orange events", () => {
        const decs = lifelines.buildLifelineDecorations([], sampleGraph);
        const ownerDecs = decs.filter((d) => d.color === "#2ecc71");
        assert.strictEqual(ownerDecs.length, 0, "Owner lifelines should be disabled");
        const moveDecs = decs.filter((d) => d.color === "#e67e22");
        assert.ok(moveDecs.length > 0, "Moves should still show as orange");
    });
    // 2. Shared borrow lifeline (blue) spans borrow to last use
    it("shared borrow lifeline is blue with correct chars", () => {
        const scopes = [{ borrower: "r", target: "data", is_mutable: false, range: { start: { line: 3 }, end: { line: 5 } } }];
        const decs = lifelines.buildLifelineDecorations(scopes);
        const blueDecs = decs.filter((d) => d.color === "#3498db");
        assert.ok(blueDecs.length === 3, `Should have 3 blue lines, got ${blueDecs.length}`);
        assert.strictEqual(blueDecs[0].char, "├─");
        assert.strictEqual(blueDecs[1].char, "│ ");
        assert.strictEqual(blueDecs[2].char, "╰─");
    });
    // 3. Mutable borrow lifeline (red) spans borrow to last use
    it("mutable borrow lifeline is red", () => {
        const scopes = [{ borrower: "m", target: "buf", is_mutable: true, range: { start: { line: 7 }, end: { line: 9 } } }];
        const decs = lifelines.buildLifelineDecorations(scopes);
        const redDecs = decs.filter((d) => d.color === "#e74c3c");
        assert.ok(redDecs.length === 3);
        assert.ok(redDecs[0].suffix.includes("&mut"), `Should mention &mut: ${redDecs[0].suffix}`);
    });
    // 4. Rc/Arc lifeline (purple) spans clone lifetime
    it("Rc clone lifeline is purple", () => {
        const decs = lifelines.buildLifelineDecorations([], sampleGraph);
        const purpleDecs = decs.filter((d) => d.color === "#9b59b6");
        assert.ok(purpleDecs.length > 0, "Should have purple Rc decorations");
        const start = purpleDecs.find((d) => d.suffix.includes("cloned"));
        assert.ok(start, "Should mention cloned");
        assert.ok(start.suffix.includes("rc2"), `Should mention clone var: ${start.suffix}`);
        assert.ok(start.suffix.includes("rc1"), `Should mention source var: ${start.suffix}`);
    });
    // 5. Conflict zone (yellow) shown when borrows overlap
    it("conflict zone is yellow with warning", () => {
        const decs = lifelines.buildLifelineDecorations([], sampleGraph);
        const yellowDecs = decs.filter((d) => d.color === "#f1c40f");
        assert.ok(yellowDecs.length > 0, "Should have yellow conflict decorations");
        const warning = yellowDecs.find((d) => d.suffix.includes("overlap"));
        assert.ok(warning, "Should mention overlap");
        assert.ok(warning.suffix.includes("r1"), `Should mention borrow_a: ${warning.suffix}`);
        assert.ok(warning.suffix.includes("m1"), `Should mention borrow_b: ${warning.suffix}`);
    });
    // 6. Move event (orange) shows maps-to arrow
    it("move event is orange with maps-to symbol", () => {
        const decs = lifelines.buildLifelineDecorations([], sampleGraph);
        const moveDecs = decs.filter((d) => d.color === "#e67e22");
        assert.ok(moveDecs.length > 0, "Should have orange move decorations");
        assert.strictEqual(moveDecs[0].char, "↦ ");
        assert.ok(moveDecs[0].suffix.includes("data") || moveDecs[0].suffix.includes("moved"), `Should mention source or moved: ${moveDecs[0].suffix}`);
        assert.ok(moveDecs[0].suffix.includes("data") || moveDecs[0].suffix.includes("⤳"), `Should mention source: ${moveDecs[0].suffix}`);
    });
    // 7. Hover messages contain variable names
    it("hover messages contain variable names", () => {
        const scopes = [{ borrower: "r", target: "data", is_mutable: false, range: { start: { line: 2 }, end: { line: 4 } } }];
        const decs = lifelines.buildLifelineDecorations(scopes);
        assert.ok(decs[0].hover.includes("r"), "Hover should mention borrower");
        assert.ok(decs[0].hover.includes("data"), "Hover should mention target");
    });
    // 8. Empty input produces no decorations
    it("empty scopes and no graph produces no decorations", () => {
        const decs = lifelines.buildLifelineDecorations([]);
        assert.strictEqual(decs.length, 0);
    });
    // 9. applyLifelines applies to editor and clearLifelines removes
    it("applyLifelines and clearLifelines work", () => {
        const editor = mockEditor();
        const scopes = [{ borrower: "r", target: "x", is_mutable: false, range: { start: { line: 0 }, end: { line: 2 } } }];
        lifelines.applyLifelines(editor, scopes);
        let total = 0;
        for (const decs of editor.__applied.values())
            total += decs.length;
        assert.ok(total > 0, "Should have applied decorations");
        lifelines.clearLifelines(editor);
        for (const decs of editor.__applied.values()) {
            assert.strictEqual(decs.length, 0);
        }
    });
    // 10. Nested borrows produce separate colored lines on same range
    it("nested borrows produce multiple decorations on overlapping lines", () => {
        const scopes = [
            { borrower: "r1", target: "data", is_mutable: false, range: { start: { line: 2 }, end: { line: 6 } } },
            { borrower: "r2", target: "data", is_mutable: false, range: { start: { line: 3 }, end: { line: 5 } } },
        ];
        const decs = lifelines.buildLifelineDecorations(scopes);
        // Line 3 should have decorations from both r1 (continuation) and r2 (start)
        const line3 = decs.filter((d) => d.line === 3);
        assert.ok(line3.length >= 2, `Line 3 should have >= 2 decorations (nested), got ${line3.length}`);
    });
});
//# sourceMappingURL=lifelines.test.js.map