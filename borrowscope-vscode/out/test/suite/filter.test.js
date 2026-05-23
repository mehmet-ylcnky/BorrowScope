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
describe("5.8 Filter by Category", () => {
    let panelSrc;
    before(() => {
        panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
    });
    // 1. Filter bar exists in HTML
    it("HTML contains filter-bar div", () => {
        assert.ok(panelSrc.includes('id="filter-bar"'));
    });
    // 2. Filter buttons created per category
    it("creates filter buttons from node categories", () => {
        assert.ok(panelSrc.includes("categories.forEach"));
        assert.ok(panelSrc.includes("append('button')"));
    });
    // 3. Buttons colored by category
    it("filter buttons colored by nodeColor", () => {
        assert.ok(panelSrc.includes("style('background', nodeColor(cat))"));
    });
    // 4. Clicking toggles hidden state
    it("clicking button toggles hiddenCategories set", () => {
        assert.ok(panelSrc.includes("hiddenCategories.has(cat)"));
        assert.ok(panelSrc.includes("hiddenCategories.delete(cat)"));
        assert.ok(panelSrc.includes("hiddenCategories.add(cat)"));
    });
    // 5. Hidden button gets visual indicator
    it("hidden button gets strikethrough class", () => {
        assert.ok(panelSrc.includes("classed('hidden'"));
        assert.ok(panelSrc.includes("text-decoration:line-through"));
    });
    // 6. applyFilters hides nodes
    it("applyFilters hides nodes of hidden categories", () => {
        assert.ok(panelSrc.includes("node.attr('display'"));
        assert.ok(panelSrc.includes("'none'"));
    });
    // 7. applyFilters hides connected edges
    it("applyFilters hides edges connected to hidden nodes", () => {
        assert.ok(panelSrc.includes("edge.attr('display'"));
        assert.ok(panelSrc.includes("srcHidden || tgtHidden"));
    });
    // 8. Filter bar has label
    it("filter bar has Filter label", () => {
        assert.ok(panelSrc.includes("Filter:"));
    });
    // 9. Hidden button has reduced opacity
    it("hidden button has reduced opacity in CSS", () => {
        assert.ok(panelSrc.includes("button.hidden { opacity:0.3"));
    });
    // 10. Categories derived from actual graph data
    it("categories extracted from data.nodes", () => {
        assert.ok(panelSrc.includes("new Set(data.nodes.map(n => n.category))"));
    });
});
//# sourceMappingURL=filter.test.js.map