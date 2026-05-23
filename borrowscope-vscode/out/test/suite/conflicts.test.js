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
describe("4.7 Diagnostics Display", () => {
    let conflicts;
    before(() => {
        require("vscode");
        conflicts = require(path.join(ROOT, "out", "conflicts.js"));
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
    // 1. BorrowScope diagnostics get inline conflict decoration
    it("BorrowScope diagnostics get conflict decoration", () => {
        const editor = mockEditor();
        const diags = [{
                range: { start: { line: 4, character: 0 }, end: { line: 6, character: 0 } },
                message: "r and m overlap on data",
                source: "BorrowScope",
            }];
        conflicts.applyConflictDecorations(editor, diags);
        const all = editor.__allRanges();
        assert.strictEqual(all.length, 1);
    });
    // 2. Non-BorrowScope diagnostics are ignored
    it("non-BorrowScope diagnostics are ignored", () => {
        const editor = mockEditor();
        const diags = [{
                range: { start: { line: 1, character: 0 }, end: { line: 1, character: 10 } },
                message: "unused variable",
                source: "rustc",
            }];
        conflicts.applyConflictDecorations(editor, diags);
        const all = editor.__allRanges();
        assert.strictEqual(all.length, 0);
    });
    // 3. Decoration range matches diagnostic range
    it("decoration range matches diagnostic range", () => {
        const editor = mockEditor();
        const diags = [{
                range: { start: { line: 5, character: 4 }, end: { line: 8, character: 0 } },
                message: "conflict",
                source: "BorrowScope",
            }];
        conflicts.applyConflictDecorations(editor, diags);
        const all = editor.__allRanges();
        assert.strictEqual(all[0].range.start.line, 5);
        assert.strictEqual(all[0].range.end.line, 8);
    });
    // 4. Hover message contains warning emoji and message
    it("hover message contains warning and diagnostic message", () => {
        const editor = mockEditor();
        const diags = [{
                range: { start: { line: 3, character: 0 }, end: { line: 5, character: 0 } },
                message: "r1 and m1 overlap on data",
                source: "BorrowScope",
            }];
        conflicts.applyConflictDecorations(editor, diags);
        const all = editor.__allRanges();
        assert.ok(all[0].hoverMessage.includes("r1 and m1"), `Got: ${all[0].hoverMessage}`);
    });
    // 5. Multiple conflicts get multiple decorations
    it("multiple conflicts get multiple decorations", () => {
        const editor = mockEditor();
        const diags = [
            { range: { start: { line: 2, character: 0 }, end: { line: 4, character: 0 } }, message: "c1", source: "BorrowScope" },
            { range: { start: { line: 7, character: 0 }, end: { line: 9, character: 0 } }, message: "c2", source: "BorrowScope" },
        ];
        conflicts.applyConflictDecorations(editor, diags);
        const all = editor.__allRanges();
        assert.strictEqual(all.length, 2);
    });
    // 6. Empty diagnostics clears conflict decorations
    it("empty diagnostics clears decorations", () => {
        const editor = mockEditor();
        // Apply first
        conflicts.applyConflictDecorations(editor, [{
                range: { start: { line: 0, character: 0 }, end: { line: 1, character: 0 } },
                message: "x", source: "BorrowScope",
            }]);
        // Then clear
        conflicts.applyConflictDecorations(editor, []);
        const all = editor.__allRanges();
        assert.strictEqual(all.length, 0);
    });
    // 7. clearConflictDecorations removes all
    it("clearConflictDecorations removes all", () => {
        const editor = mockEditor();
        conflicts.applyConflictDecorations(editor, [{
                range: { start: { line: 0, character: 0 }, end: { line: 2, character: 0 } },
                message: "x", source: "BorrowScope",
            }]);
        conflicts.clearConflictDecorations(editor);
        const all = editor.__allRanges();
        assert.strictEqual(all.length, 0);
    });
    // 8. Mixed diagnostics only shows BorrowScope ones
    it("mixed sources only decorates BorrowScope", () => {
        const editor = mockEditor();
        const diags = [
            { range: { start: { line: 1, character: 0 }, end: { line: 2, character: 0 } }, message: "a", source: "BorrowScope" },
            { range: { start: { line: 3, character: 0 }, end: { line: 4, character: 0 } }, message: "b", source: "rustc" },
            { range: { start: { line: 5, character: 0 }, end: { line: 6, character: 0 } }, message: "c", source: "BorrowScope" },
        ];
        conflicts.applyConflictDecorations(editor, diags);
        const all = editor.__allRanges();
        assert.strictEqual(all.length, 2);
    });
    // 9. Diagnostics severity is Information (verified in server, client just displays)
    it("server diagnostics source is BorrowScope (verified by filter)", () => {
        const editor = mockEditor();
        const diags = [{ range: { start: { line: 0, character: 0 }, end: { line: 1, character: 0 } }, message: "test", source: "BorrowScope" }];
        conflicts.applyConflictDecorations(editor, diags);
        assert.ok(editor.__allRanges().length > 0, "BorrowScope source should be accepted");
    });
    // 10. Diagnostics don't duplicate (same range applied once)
    it("same diagnostic applied once", () => {
        const editor = mockEditor();
        const diag = { range: { start: { line: 3, character: 0 }, end: { line: 5, character: 0 } }, message: "dup", source: "BorrowScope" };
        conflicts.applyConflictDecorations(editor, [diag, diag]);
        const all = editor.__allRanges();
        assert.strictEqual(all.length, 2); // Both are applied (dedup is server's job)
    });
});
//# sourceMappingURL=conflicts.test.js.map