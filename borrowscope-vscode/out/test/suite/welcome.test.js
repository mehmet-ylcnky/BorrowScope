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
describe("8.5 Welcome View and Onboarding", () => {
    let welcome;
    let pkgJson;
    let extSrc;
    before(() => {
        welcome = require(path.join(ROOT, "out", "welcome"));
        pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
        extSrc = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
    });
    // === Module exports ===
    it("exports showWelcomeIfNeeded", () => {
        assert.strictEqual(typeof welcome.showWelcomeIfNeeded, "function");
    });
    it("exports showWelcomePanel", () => {
        assert.strictEqual(typeof welcome.showWelcomePanel, "function");
    });
    it("exports checkPrerequisites", () => {
        assert.strictEqual(typeof welcome.checkPrerequisites, "function");
    });
    // === checkPrerequisites ===
    it("checkPrerequisites returns PrerequisiteStatus", async () => {
        const status = await welcome.checkPrerequisites();
        assert.ok("rustToolchain" in status);
        assert.ok("rustVersion" in status);
        assert.ok("cargoProject" in status);
        assert.ok("projectName" in status);
        assert.ok("serverBinary" in status);
    });
    it("checkPrerequisites detects rust toolchain", async () => {
        const status = await welcome.checkPrerequisites();
        // In test environment, rustc should be available
        assert.strictEqual(typeof status.rustToolchain, "boolean");
    });
    // === Integration ===
    it("extension.ts imports welcome module", () => {
        assert.ok(extSrc.includes("showWelcomeIfNeeded"));
        assert.ok(extSrc.includes("./welcome"));
    });
    it("extension.ts calls showWelcomeIfNeeded on activation", () => {
        assert.ok(extSrc.includes("showWelcomeIfNeeded(context)"));
    });
    it("showWelcome command is registered", () => {
        assert.ok(extSrc.includes("borrowscope.showWelcome"));
    });
    it("package.json has showWelcome command", () => {
        assert.ok(pkgJson.contributes.commands.some((c) => c.command === "borrowscope.showWelcome"));
    });
    // === Welcome HTML content ===
    it("welcome source has prerequisite checks", () => {
        const src = fs.readFileSync(path.join(ROOT, "src", "welcome.ts"), "utf8");
        assert.ok(src.includes("rustc --version"));
        assert.ok(src.includes("Cargo.toml"));
        assert.ok(src.includes("server.path"));
    });
    it("welcome source has getting started steps", () => {
        const src = fs.readFileSync(path.join(ROOT, "src", "welcome.ts"), "utf8");
        assert.ok(src.includes("Getting Started"));
        assert.ok(src.includes("Ctrl+Shift+O"));
    });
    it("welcome source has action buttons", () => {
        const src = fs.readFileSync(path.join(ROOT, "src", "welcome.ts"), "utf8");
        assert.ok(src.includes("openRustFile"));
        assert.ok(src.includes("showGraph"));
        assert.ok(src.includes("openDocs"));
        assert.ok(src.includes("dismiss"));
    });
    it("welcome uses globalState to track shown status", () => {
        const src = fs.readFileSync(path.join(ROOT, "src", "welcome.ts"), "utf8");
        assert.ok(src.includes("borrowscope.welcomeShown"));
        assert.ok(src.includes("globalState.get"));
        assert.ok(src.includes("globalState.update"));
    });
    it("welcome does not show if already shown", () => {
        const src = fs.readFileSync(path.join(ROOT, "src", "welcome.ts"), "utf8");
        assert.ok(src.includes("if (hasShown) return"));
    });
});
//# sourceMappingURL=welcome.test.js.map