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
describe("4.6 Borrow Scope Background Highlighting", () => {
    let highlights;
    before(() => {
        require("vscode");
        highlights = require(path.join(ROOT, "out", "highlights.js"));
    });
    function mockEditor() {
        const applied = new Map();
        return {
            setDecorations: (dt, ranges) => { applied.set(dt, ranges); },
            __applied: applied,
            __allRanges: () => {
                const all = [];
                for (const decs of applied.values())
                    all.push(...decs);
                return all;
            },
        };
    }
    // 1. Shared borrow gets blue background
    it("shared borrow region gets blue background", () => {
        const editor = mockEditor();
        const scopes = [{ borrower: "r", target: "data", is_mutable: false, range: { start: { line: 2 }, end: { line: 5 } } }];
        highlights.applyHighlights(editor, scopes);
        const all = editor.__allRanges();
        assert.ok(all.length > 0, "Should apply highlight decorations");
        assert.ok(all[0].hoverMessage.includes("&"), "Should mention shared borrow");
    });
    // 2. Mutable borrow gets red background
    it("mutable borrow region gets red background", () => {
        const editor = mockEditor();
        const scopes = [{ borrower: "m", target: "buf", is_mutable: true, range: { start: { line: 3 }, end: { line: 6 } } }];
        highlights.applyHighlights(editor, scopes);
        const all = editor.__allRanges();
        assert.ok(all.length > 0);
        assert.ok(all[0].hoverMessage.includes("&mut"), "Should mention mutable borrow");
    });
    // 3. Highlighting spans from borrow creation to last use
    it("highlight range matches borrow scope range", () => {
        const editor = mockEditor();
        const scopes = [{ borrower: "r", target: "x", is_mutable: false, range: { start: { line: 4 }, end: { line: 8 } } }];
        highlights.applyHighlights(editor, scopes);
        const all = editor.__allRanges();
        assert.strictEqual(all[0].range.start.line, 4);
        assert.strictEqual(all[0].range.end.line, 8);
    });
    // 4. Multiple borrows each get their own highlight
    it("multiple borrows get separate highlights", () => {
        const editor = mockEditor();
        const scopes = [
            { borrower: "r1", target: "data", is_mutable: false, range: { start: { line: 2 }, end: { line: 4 } } },
            { borrower: "r2", target: "data", is_mutable: false, range: { start: { line: 3 }, end: { line: 5 } } },
            { borrower: "m", target: "buf", is_mutable: true, range: { start: { line: 7 }, end: { line: 9 } } },
        ];
        highlights.applyHighlights(editor, scopes);
        const all = editor.__allRanges();
        assert.strictEqual(all.length, 3, `Should have 3 highlights, got ${all.length}`);
    });
    // 5. Conflict zone gets yellow background
    it("conflict zone gets yellow highlight", () => {
        const editor = mockEditor();
        const graph = {
            conflicts: [{ variable: "data", borrow_a: "r", borrow_b: "m", overlap_start_line: 4, overlap_end_line: 6 }],
        };
        highlights.applyHighlights(editor, [], graph);
        const all = editor.__allRanges();
        assert.ok(all.length > 0, "Should have conflict highlight");
        assert.ok(all[0].hoverMessage.includes("Conflict"), "Should mention conflict");
    });
    // 6. No highlights for empty scopes
    it("empty scopes produces no highlights", () => {
        const editor = mockEditor();
        highlights.applyHighlights(editor, []);
        const all = editor.__allRanges();
        assert.strictEqual(all.length, 0);
    });
    // 7. clearHighlights removes all
    it("clearHighlights removes all highlights", () => {
        const editor = mockEditor();
        const scopes = [{ borrower: "r", target: "x", is_mutable: false, range: { start: { line: 0 }, end: { line: 3 } } }];
        highlights.applyHighlights(editor, scopes);
        highlights.clearHighlights(editor);
        const all = editor.__allRanges();
        assert.strictEqual(all.length, 0);
    });
    // 8. isHighlightsEnabled returns true by default
    it("isHighlightsEnabled returns true by default", () => {
        assert.strictEqual(highlights.isHighlightsEnabled(), true);
    });
    // 9. Hover message contains variable names
    it("hover message contains borrower and target names", () => {
        const editor = mockEditor();
        const scopes = [{ borrower: "reader", target: "buffer", is_mutable: false, range: { start: { line: 1 }, end: { line: 3 } } }];
        highlights.applyHighlights(editor, scopes);
        const all = editor.__allRanges();
        assert.ok(all[0].hoverMessage.includes("reader"));
        assert.ok(all[0].hoverMessage.includes("buffer"));
    });
    // 10. Shared and mutable go to different decoration types
    it("shared and mutable borrows use different decoration types", () => {
        const editor = mockEditor();
        const scopes = [
            { borrower: "r", target: "x", is_mutable: false, range: { start: { line: 1 }, end: { line: 2 } } },
            { borrower: "m", target: "x", is_mutable: true, range: { start: { line: 4 }, end: { line: 5 } } },
        ];
        highlights.applyHighlights(editor, scopes);
        // Should have applied to at least 2 different decoration types
        assert.ok(editor.__applied.size >= 2, `Should use >= 2 decoration types, got ${editor.__applied.size}`);
    });
});
//# sourceMappingURL=highlights.test.js.map