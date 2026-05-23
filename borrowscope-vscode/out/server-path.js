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
exports.resolveServerPath = resolveServerPath;
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
function resolveServerPath(ctx) {
    // 1. User-configured path
    if (ctx.configuredPath && fs.existsSync(ctx.configuredPath)) {
        return ctx.configuredPath;
    }
    // 2. Bundled binary
    const bundled = path.join(ctx.extensionPath, "server", "borrowscope-lsp");
    if (fs.existsSync(bundled)) {
        return bundled;
    }
    // 3. Downloaded binary in global storage
    if (ctx.globalStoragePath) {
        const binaryName = process.platform === "win32" ? "borrowscope-lsp.exe" : "borrowscope-lsp";
        const downloaded = path.join(ctx.globalStoragePath, binaryName);
        if (fs.existsSync(downloaded)) {
            return downloaded;
        }
    }
    // 4. System PATH
    const pathDirs = (process.env.PATH || "").split(path.delimiter);
    for (const dir of pathDirs) {
        const candidate = path.join(dir, "borrowscope-lsp");
        if (fs.existsSync(candidate)) {
            return candidate;
        }
    }
    throw new Error("borrowscope-lsp binary not found. Install it or set borrowscope.server.path.");
}
//# sourceMappingURL=server-path.js.map