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
describe("11.4 Cross-Function Lifeline Rendering", () => {
    let crossLifelines;
    let clientSrc;
    before(() => {
        crossLifelines = require(path.join(ROOT, "out", "cross-lifelines.js"));
        clientSrc = fs.readFileSync(path.join(ROOT, "src", "client.ts"), "utf8");
    });
    function mockEditor(lines) {
        const applied = new Map();
        return {
            document: {
                lineCount: lines.length,
                lineAt: (i) => ({ text: lines[i] || "" }),
                uri: { toString: () => "file:///test.rs" },
            },
            selection: { active: { line: 0 } },
            setDecorations: (dt, decs) => { applied.set(dt, decs); },
            __applied: applied,
            __allDecs: () => { const all = []; for (const d of applied.values())
                all.push(...d); return all; },
        };
    }
    // 1. Dashed lines drawn between caller and callee
    it("draws dashed connection lines between functions", () => {
        const editor = mockEditor([
            "fn caller() {",
            "    process(&data);",
            "}",
            "",
            "fn process(items: &[i32]) {",
            "    println!(\"{:?}\", items);",
            "}",
        ]);
        const borrows = [{
                origin_variable: "data",
                origin_line: 2,
                path: [
                    { file: "test.rs", function_name: "caller", variable: "data", start_line: 2, end_line: 2, is_mutable: false, kind: "Origin" },
                    { file: "test.rs", function_name: "process", variable: "items", start_line: 5, end_line: 6, is_mutable: false, kind: "Parameter" },
                ],
            }];
        crossLifelines.applyCrossLifelines(editor, borrows);
        const decs = editor.__allDecs();
        // Should have dashed lines between line 1 (call) and line 4 (callee)
        const dashedLines = decs.filter((d) => d.renderOptions?.before?.contentText?.includes("┊"));
        assert.ok(dashedLines.length > 0, `Should have dashed connection lines. Got ${decs.length} total decorations`);
    });
    // 2. Callee gets "received from" annotation
    it("callee function gets received annotation", () => {
        const editor = mockEditor([
            "fn caller() {",
            "    process(&data);",
            "}",
            "",
            "fn process(items: &[i32]) {",
            "}",
        ]);
        const borrows = [{
                origin_variable: "data",
                origin_line: 2,
                path: [
                    { file: "test.rs", function_name: "caller", variable: "data", start_line: 2, end_line: 2, is_mutable: false, kind: "Origin" },
                    { file: "test.rs", function_name: "process", variable: "items", start_line: 5, end_line: 5, is_mutable: false, kind: "Parameter" },
                ],
            }];
        crossLifelines.applyCrossLifelines(editor, borrows);
        const decs = editor.__allDecs();
        const received = decs.find((d) => d.renderOptions?.after?.contentText?.includes("receives"));
        assert.ok(received, "Callee should have 'receives' annotation");
    });
    // 3. Mutable borrows show lock icon
    it("mutable cross-function borrow shows lock icon", () => {
        const editor = mockEditor([
            "fn caller() {",
            "    modify(&mut data);",
            "}",
            "fn modify(d: &mut Vec<i32>) {",
            "}",
        ]);
        const borrows = [{
                origin_variable: "data",
                origin_line: 2,
                path: [
                    { file: "test.rs", function_name: "caller", variable: "data", start_line: 2, end_line: 2, is_mutable: true, kind: "Origin" },
                    { file: "test.rs", function_name: "modify", variable: "d", start_line: 4, end_line: 4, is_mutable: true, kind: "Parameter" },
                ],
            }];
        crossLifelines.applyCrossLifelines(editor, borrows);
        const decs = editor.__allDecs();
        const lockDec = decs.find((d) => d.renderOptions?.after?.contentText?.includes("🔒"));
        assert.ok(lockDec, "Mutable borrow should show lock icon");
    });
    // 4. No decorations when callee not in same file
    it("no decorations when callee function not found in file", () => {
        const editor = mockEditor(["fn caller() {", "    external_fn(&data);", "}"]);
        const borrows = [{
                origin_variable: "data",
                origin_line: 2,
                path: [
                    { file: "test.rs", function_name: "caller", variable: "data", start_line: 2, end_line: 2, is_mutable: false, kind: "Origin" },
                    { file: "other.rs", function_name: "external_fn", variable: "x", start_line: 1, end_line: 5, is_mutable: false, kind: "Parameter" },
                ],
            }];
        crossLifelines.applyCrossLifelines(editor, borrows);
        const decs = editor.__allDecs();
        assert.strictEqual(decs.length, 0, "Should not render for functions not in current file");
    });
    // 5. Empty borrows produces no decorations
    it("empty borrows produces no decorations", () => {
        const editor = mockEditor(["fn test() {}"]);
        crossLifelines.applyCrossLifelines(editor, []);
        assert.strictEqual(editor.__allDecs().length, 0);
    });
    // 6. clearCrossLifelines removes all
    it("clearCrossLifelines removes decorations", () => {
        const editor = mockEditor(["fn a() {}", "fn b(x: &i32) {}"]);
        crossLifelines.applyCrossLifelines(editor, [{
                origin_variable: "v", origin_line: 1,
                path: [
                    { file: "test.rs", function_name: "a", variable: "v", start_line: 1, end_line: 1, is_mutable: false, kind: "Origin" },
                    { file: "test.rs", function_name: "b", variable: "x", start_line: 2, end_line: 2, is_mutable: false, kind: "Parameter" },
                ],
            }]);
        crossLifelines.clearCrossLifelines(editor);
        assert.strictEqual(editor.__allDecs().length, 0);
    });
    // 7. Hover message shows borrow details
    it("hover message shows cross-function borrow details", () => {
        const editor = mockEditor(["fn a() {}", "fn b(x: &i32) {}"]);
        crossLifelines.applyCrossLifelines(editor, [{
                origin_variable: "data", origin_line: 1,
                path: [
                    { file: "test.rs", function_name: "a", variable: "data", start_line: 1, end_line: 1, is_mutable: false, kind: "Origin" },
                    { file: "test.rs", function_name: "b", variable: "x", start_line: 2, end_line: 2, is_mutable: false, kind: "Parameter" },
                ],
            }]);
        const decs = editor.__allDecs();
        const withHover = decs.find((d) => d.hoverMessage);
        assert.ok(withHover, "Should have hover message");
        assert.ok(withHover.hoverMessage.includes("data"), "Hover should mention origin variable");
        assert.ok(withHover.hoverMessage.includes("b"), "Hover should mention callee");
    });
    // 8. Extension fetches crossFunctionBorrows in refreshDecorations
    it("client.ts fetches crossFunctionBorrows", () => {
        assert.ok(clientSrc.includes("borrowscope/crossFunctionBorrows"));
        assert.ok(clientSrc.includes("crossFunctionBorrows"));
    });
    // 9. Connection uses correct color for shared vs mutable
    it("shared borrow uses teal color", () => {
        const editor = mockEditor(["fn a() {}", "fn b(x: &i32) {}"]);
        crossLifelines.applyCrossLifelines(editor, [{
                origin_variable: "v", origin_line: 1,
                path: [
                    { file: "test.rs", function_name: "a", variable: "v", start_line: 1, end_line: 1, is_mutable: false, kind: "Origin" },
                    { file: "test.rs", function_name: "b", variable: "x", start_line: 2, end_line: 2, is_mutable: false, kind: "Parameter" },
                ],
            }]);
        const decs = editor.__allDecs();
        const colored = decs.find((d) => d.renderOptions?.after?.color?.includes("26, 188, 156"));
        assert.ok(colored, "Shared borrow should use teal color");
    });
    // 10. Multiple cross-function borrows rendered independently
    it("multiple borrows each get their own connection", () => {
        const editor = mockEditor(["fn a() {}", "", "fn b(x: &i32) {}", "", "fn c(y: &str) {}"]);
        crossLifelines.applyCrossLifelines(editor, [
            { origin_variable: "v1", origin_line: 1, path: [
                    { file: "test.rs", function_name: "a", variable: "v1", start_line: 1, end_line: 1, is_mutable: false, kind: "Origin" },
                    { file: "test.rs", function_name: "b", variable: "x", start_line: 3, end_line: 3, is_mutable: false, kind: "Parameter" },
                ] },
            { origin_variable: "v2", origin_line: 1, path: [
                    { file: "test.rs", function_name: "a", variable: "v2", start_line: 1, end_line: 1, is_mutable: false, kind: "Origin" },
                    { file: "test.rs", function_name: "c", variable: "y", start_line: 5, end_line: 5, is_mutable: false, kind: "Parameter" },
                ] },
        ]);
        const decs = editor.__allDecs();
        const received = decs.filter((d) => d.renderOptions?.after?.contentText?.includes("receives"));
        assert.strictEqual(received.length, 2, "Should have 2 'receives' annotations");
    });
});
//# sourceMappingURL=cross-lifelines.test.js.map