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
describe("5.2 Graph Data Model", () => {
    let model;
    before(() => {
        model = require(path.join(ROOT, "out", "graph", "model.js"));
    });
    function sampleData(overrides = {}) {
        return {
            function_name: "test_fn",
            variables: [
                { name: "data", type_display: "Vec<i32>", ownership_category: "Owned", line: 2, is_copy: false },
                { name: "r", type_display: "&Vec<i32>", ownership_category: "SharedRef", line: 3, is_copy: false },
            ],
            borrow_scopes: [
                { borrower_name: "r", target_name: "data", is_mutable: false, start_line: 3, end_line: 5 },
            ],
            moves: [],
            rc_clones: [],
            ...overrides,
        };
    }
    // 1. buildGraphModel returns correct structure
    it("returns GraphModel with functionName, nodes, edges", () => {
        const result = model.buildGraphModel(sampleData());
        assert.strictEqual(result.metadata.functionName, "test_fn");
        assert.ok(Array.isArray(result.nodes));
        assert.ok(Array.isArray(result.edges));
    });
    // 2. Nodes created from variables
    it("creates nodes from variables", () => {
        const result = model.buildGraphModel(sampleData());
        assert.strictEqual(result.nodes.length, 2);
        assert.strictEqual(result.nodes[0].id, "data");
        assert.strictEqual(result.nodes[1].id, "r");
    });
    // 3. Node has all required fields
    it("nodes have id, name, type, category, line, isCopy", () => {
        const result = model.buildGraphModel(sampleData());
        const node = result.nodes[0];
        assert.strictEqual(node.id, "data");
        assert.strictEqual(node.name, "data");
        assert.strictEqual(node.type, "Vec<i32>");
        assert.strictEqual(node.category, "Owned");
        assert.strictEqual(node.line, 2);
        assert.strictEqual(node.isCopy, false);
    });
    // 4. Shared borrow creates edge
    it("shared borrow creates shared_borrow edge", () => {
        const result = model.buildGraphModel(sampleData());
        assert.strictEqual(result.edges.length, 1);
        assert.strictEqual(result.edges[0].source, "data");
        assert.strictEqual(result.edges[0].target, "r");
        assert.strictEqual(result.edges[0].kind, "shared_borrow");
        assert.strictEqual(result.edges[0].isMutable, false);
        assert.strictEqual(result.edges[0].label, "&");
    });
    // 5. Mutable borrow creates mutable_borrow edge
    it("mutable borrow creates mutable_borrow edge", () => {
        const data = sampleData({
            borrow_scopes: [{ borrower_name: "m", target_name: "data", is_mutable: true, start_line: 4, end_line: 6 }],
            variables: [
                { name: "data", type_display: "Vec<i32>", ownership_category: "Owned", line: 2, is_copy: false },
                { name: "m", type_display: "&mut Vec<i32>", ownership_category: "MutableRef", line: 4, is_copy: false },
            ],
        });
        const result = model.buildGraphModel(data);
        const edge = result.edges.find((e) => e.target === "m");
        assert.ok(edge);
        assert.strictEqual(edge.kind, "mutable_borrow");
        assert.strictEqual(edge.isMutable, true);
        assert.strictEqual(edge.label, "&mut");
    });
    // 6. Move creates move edge
    it("move creates move edge", () => {
        const data = sampleData({
            borrow_scopes: [],
            moves: [{ source_name: "a", destination: "b", line: 5, source_type: "String" }],
            variables: [
                { name: "a", type_display: "String", ownership_category: "Owned", line: 2, is_copy: false },
                { name: "b", type_display: "String", ownership_category: "Owned", line: 5, is_copy: false },
            ],
        });
        const result = model.buildGraphModel(data);
        const edge = result.edges.find((e) => e.kind === "move");
        assert.ok(edge, "Should have move edge");
        assert.strictEqual(edge.source, "a");
        assert.strictEqual(edge.target, "b");
        assert.strictEqual(edge.label, "move");
    });
    // 7. Rc clone creates rc_clone edge
    it("Rc clone creates rc_clone edge", () => {
        const data = sampleData({
            borrow_scopes: [],
            rc_clones: [{ clone_variable: "rc2", source_variable: "rc1", clone_type: "Rc", line: 4 }],
            variables: [
                { name: "rc1", type_display: "Rc<i32>", ownership_category: "Rc", line: 2, is_copy: false },
                { name: "rc2", type_display: "Rc<i32>", ownership_category: "Rc", line: 4, is_copy: false },
            ],
        });
        const result = model.buildGraphModel(data);
        const edge = result.edges.find((e) => e.kind === "rc_clone");
        assert.ok(edge);
        assert.strictEqual(edge.source, "rc1");
        assert.strictEqual(edge.target, "rc2");
        assert.strictEqual(edge.label, "Rc::clone");
    });
    // 8. Arc clone creates arc_clone edge
    it("Arc clone creates arc_clone edge", () => {
        const data = sampleData({
            borrow_scopes: [],
            rc_clones: [{ clone_variable: "arc2", source_variable: "arc1", clone_type: "Arc", line: 4 }],
            variables: [
                { name: "arc1", type_display: "Arc<i32>", ownership_category: "Arc", line: 2, is_copy: false },
                { name: "arc2", type_display: "Arc<i32>", ownership_category: "Arc", line: 4, is_copy: false },
            ],
        });
        const result = model.buildGraphModel(data);
        const edge = result.edges.find((e) => e.kind === "arc_clone");
        assert.ok(edge);
        assert.strictEqual(edge.label, "Arc::clone");
    });
    // 9. Missing nodes are auto-created for edges
    it("auto-creates nodes referenced in edges but not in variables", () => {
        const data = {
            function_name: "f",
            variables: [{ name: "data", type_display: "Vec<i32>", ownership_category: "Owned", line: 2, is_copy: false }],
            borrow_scopes: [{ borrower_name: "unknown_ref", target_name: "data", is_mutable: false, start_line: 3, end_line: 5 }],
            moves: [],
            rc_clones: [],
        };
        const result = model.buildGraphModel(data);
        const unknownNode = result.nodes.find((n) => n.id === "unknown_ref");
        assert.ok(unknownNode, "Should auto-create missing node");
        assert.strictEqual(unknownNode.category, "Unknown");
    });
    // 10. Duplicate edges are removed
    it("deduplicates identical edges", () => {
        const data = sampleData({
            borrow_scopes: [
                { borrower_name: "r", target_name: "data", is_mutable: false, start_line: 3, end_line: 5 },
                { borrower_name: "r", target_name: "data", is_mutable: false, start_line: 3, end_line: 5 },
            ],
        });
        const result = model.buildGraphModel(data);
        const borrowEdges = result.edges.filter((e) => e.kind === "shared_borrow");
        assert.strictEqual(borrowEdges.length, 1, "Should deduplicate");
    });
    // 11. Empty data produces empty graph
    it("empty data produces empty graph", () => {
        const data = { function_name: "empty", variables: [], borrow_scopes: [], moves: [], rc_clones: [] };
        const result = model.buildGraphModel(data);
        assert.strictEqual(result.nodes.length, 0);
        assert.strictEqual(result.edges.length, 0);
        assert.strictEqual(result.metadata.functionName, "empty");
    });
    // 12. Duplicate variable names don't create duplicate nodes
    it("duplicate variable names produce single node", () => {
        const data = sampleData({
            variables: [
                { name: "x", type_display: "i32", ownership_category: "Copy", line: 2, is_copy: true },
                { name: "x", type_display: "i32", ownership_category: "Copy", line: 2, is_copy: true },
            ],
            borrow_scopes: [],
        });
        const result = model.buildGraphModel(data);
        assert.strictEqual(result.nodes.length, 1);
    });
    // 13. Move destination as object (FunctionArg)
    it("handles move destination as FunctionArg object", () => {
        const data = sampleData({
            borrow_scopes: [],
            moves: [{ source_name: "v", destination: { FunctionArg: { function: "drop" } }, line: 5, source_type: "Vec<i32>" }],
        });
        const result = model.buildGraphModel(data);
        const edge = result.edges.find((e) => e.kind === "move");
        assert.ok(edge);
        assert.strictEqual(edge.target, "drop()");
    });
    // 14. Move destination as Return
    it("handles move destination as Return", () => {
        const data = sampleData({
            borrow_scopes: [],
            moves: [{ source_name: "result", destination: { Return: true }, line: 10, source_type: "String" }],
        });
        const result = model.buildGraphModel(data);
        const edge = result.edges.find((e) => e.kind === "move");
        assert.ok(edge);
        assert.strictEqual(edge.target, "(return)");
    });
    // 15. Move destination as Closure
    it("handles move destination as Closure", () => {
        const data = sampleData({
            borrow_scopes: [],
            moves: [{ source_name: "name", destination: { Closure: true }, line: 8, source_type: "String" }],
        });
        const result = model.buildGraphModel(data);
        const edge = result.edges.find((e) => e.kind === "move");
        assert.ok(edge);
        assert.strictEqual(edge.target, "(closure)");
    });
    // 16. Multiple borrows from same source
    it("multiple borrows from same source create multiple edges", () => {
        const data = {
            function_name: "f",
            variables: [
                { name: "data", type_display: "Vec<i32>", ownership_category: "Owned", line: 2, is_copy: false },
                { name: "r1", type_display: "&Vec<i32>", ownership_category: "SharedRef", line: 3, is_copy: false },
                { name: "r2", type_display: "&Vec<i32>", ownership_category: "SharedRef", line: 4, is_copy: false },
            ],
            borrow_scopes: [
                { borrower_name: "r1", target_name: "data", is_mutable: false, start_line: 3, end_line: 5 },
                { borrower_name: "r2", target_name: "data", is_mutable: false, start_line: 4, end_line: 6 },
            ],
            moves: [],
            rc_clones: [],
        };
        const result = model.buildGraphModel(data);
        const borrowEdges = result.edges.filter((e) => e.source === "data");
        assert.strictEqual(borrowEdges.length, 2);
    });
    // 17. Copy variable has isCopy=true
    it("copy variable has isCopy flag", () => {
        const data = sampleData({
            variables: [{ name: "x", type_display: "i32", ownership_category: "Copy", line: 2, is_copy: true }],
            borrow_scopes: [],
        });
        const result = model.buildGraphModel(data);
        assert.strictEqual(result.nodes[0].isCopy, true);
    });
    // 18. Complex graph with all edge types
    it("complex graph with borrows, moves, and clones", () => {
        const data = {
            function_name: "complex",
            variables: [
                { name: "data", type_display: "Vec<i32>", ownership_category: "Owned", line: 2, is_copy: false },
                { name: "r", type_display: "&Vec<i32>", ownership_category: "SharedRef", line: 3, is_copy: false },
                { name: "rc1", type_display: "Rc<i32>", ownership_category: "Rc", line: 5, is_copy: false },
                { name: "rc2", type_display: "Rc<i32>", ownership_category: "Rc", line: 6, is_copy: false },
            ],
            borrow_scopes: [{ borrower_name: "r", target_name: "data", is_mutable: false, start_line: 3, end_line: 4 }],
            moves: [{ source_name: "data", destination: "consumed", line: 8, source_type: "Vec<i32>" }],
            rc_clones: [{ clone_variable: "rc2", source_variable: "rc1", clone_type: "Rc", line: 6 }],
        };
        const result = model.buildGraphModel(data);
        assert.ok(result.nodes.length >= 4);
        assert.strictEqual(result.edges.filter((e) => e.kind === "shared_borrow").length, 1);
        assert.strictEqual(result.edges.filter((e) => e.kind === "move").length, 1);
        assert.strictEqual(result.edges.filter((e) => e.kind === "rc_clone").length, 1);
    });
    // 19. buildGraphModel is exported
    it("buildGraphModel is a function", () => {
        assert.strictEqual(typeof model.buildGraphModel, "function");
    });
    // 20. Move destination as plain string
    it("handles move destination as plain string", () => {
        const data = sampleData({
            borrow_scopes: [],
            moves: [{ source_name: "a", destination: "b", line: 5, source_type: "String" }],
        });
        const result = model.buildGraphModel(data);
        const edge = result.edges.find((e) => e.kind === "move");
        assert.strictEqual(edge.target, "b");
    });
    // 21. Node size scales with edge count
    it("node size scales with edge count", () => {
        const data = {
            function_name: "f",
            variables: [
                { name: "data", type_display: "Vec<i32>", ownership_category: "Owned", line: 2, is_copy: false },
                { name: "r1", type_display: "&Vec<i32>", ownership_category: "SharedRef", line: 3, is_copy: false },
                { name: "r2", type_display: "&Vec<i32>", ownership_category: "SharedRef", line: 4, is_copy: false },
                { name: "lonely", type_display: "i32", ownership_category: "Copy", line: 5, is_copy: true },
            ],
            borrow_scopes: [
                { borrower_name: "r1", target_name: "data", is_mutable: false, start_line: 3, end_line: 5 },
                { borrower_name: "r2", target_name: "data", is_mutable: false, start_line: 4, end_line: 6 },
            ],
            moves: [],
            rc_clones: [],
        };
        const result = model.buildGraphModel(data);
        const dataNode = result.nodes.find((n) => n.id === "data");
        const lonelyNode = result.nodes.find((n) => n.id === "lonely");
        assert.ok(dataNode.size > lonelyNode.size, `data (${dataNode.size}) should be larger than lonely (${lonelyNode.size})`);
    });
    // 22. Metadata includes stats
    it("metadata includes stats from source data", () => {
        const data = sampleData();
        const result = model.buildGraphModel(data);
        assert.strictEqual(result.metadata.stats.variables, 2);
        assert.strictEqual(result.metadata.stats.borrows, 1);
        assert.strictEqual(result.metadata.stats.moves, 0);
    });
});
//# sourceMappingURL=model.test.js.map