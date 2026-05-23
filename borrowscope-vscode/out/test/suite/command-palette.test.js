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
describe("8.3 Command Palette", () => {
    let pkgJson;
    let extSrc;
    before(() => {
        pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
        extSrc = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
    });
    const commands = () => pkgJson.contributes.commands;
    // === All spec commands exist ===
    const requiredCommands = [
        "borrowscope.showGraph",
        "borrowscope.inspectVariable",
        "borrowscope.toggleDecorations",
        "borrowscope.toggleBorrowScopes",
        "borrowscope.toggleGutterIcons",
        "borrowscope.toggleLifelines",
        "borrowscope.toggleCodeLens",
        "borrowscope.showTimeline",
        "borrowscope.showScopes",
        "borrowscope.showRefCount",
        "borrowscope.showMoves",
        "borrowscope.nextConflict",
        "borrowscope.prevConflict",
        "borrowscope.restartServer",
        "borrowscope.showServerOutput",
        "borrowscope.exportDot",
        "borrowscope.exportSvg",
        "borrowscope.focusGraph",
        "borrowscope.toggleRuntime",
    ];
    for (const cmd of requiredCommands) {
        it(`command "${cmd}" exists`, () => {
            assert.ok(commands().some((c) => c.command === cmd), `Missing command: ${cmd}`);
        });
    }
    // === All commands prefixed with "BorrowScope:" ===
    it("all commands have BorrowScope: prefix in title", () => {
        for (const cmd of commands()) {
            assert.ok(cmd.title.startsWith("BorrowScope:"), `${cmd.command} title should start with "BorrowScope:"`);
        }
    });
    // === Implementation checks ===
    it("toggleBorrowScopes toggles the setting", () => {
        assert.ok(extSrc.includes("decorations.borrowScopes"));
    });
    it("toggleGutterIcons toggles the setting", () => {
        assert.ok(extSrc.includes("decorations.gutterIcons"));
    });
    it("showTimeline opens panel with timeline view", () => {
        assert.ok(extSrc.includes('showPanelView("timeline")'));
    });
    it("showScopes opens panel with scopes view", () => {
        assert.ok(extSrc.includes('showPanelView("scopes")'));
    });
    it("showRefCount opens panel with refcount view", () => {
        assert.ok(extSrc.includes('showPanelView("refcount")'));
    });
    it("showMoves opens panel with moves view", () => {
        assert.ok(extSrc.includes('showPanelView("moves")'));
    });
    it("showServerOutput shows output channel", () => {
        assert.ok(extSrc.includes("function showServerOutput"));
        assert.ok(extSrc.includes("outputChannel.show()"));
    });
    it("exportDot generates DOT format", () => {
        assert.ok(extSrc.includes("function exportDot"));
        assert.ok(extSrc.includes("digraph"));
        assert.ok(extSrc.includes("rankdir"));
    });
    it("exportSvg generates SVG format", () => {
        assert.ok(extSrc.includes("function exportSvg"));
        assert.ok(extSrc.includes("<svg"));
        assert.ok(extSrc.includes("xmlns"));
    });
    it("export commands show error when no graph", () => {
        assert.ok(extSrc.includes("No graph data"));
    });
    // === Total command count ===
    it("has at least 20 commands", () => {
        assert.ok(commands().length >= 20, `Expected >= 20 commands, got ${commands().length}`);
    });
});
//# sourceMappingURL=command-palette.test.js.map