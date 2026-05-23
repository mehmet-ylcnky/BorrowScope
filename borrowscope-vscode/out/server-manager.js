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
exports.getPlatformAsset = getPlatformAsset;
exports.getBinaryPath = getBinaryPath;
exports.ensureServer = ensureServer;
exports.getLocalVersion = getLocalVersion;
exports.getLatestReleaseUrl = getLatestReleaseUrl;
exports.downloadServer = downloadServer;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const https = __importStar(require("https"));
const http = __importStar(require("http"));
const GITHUB_REPO = "mehmet-ylcnky/BorrowScope";
const BINARY_NAME = process.platform === "win32" ? "borrowscope-lsp.exe" : "borrowscope-lsp";
function getPlatformAsset() {
    const platform = process.platform; // linux, darwin, win32
    const arch = process.arch; // x64, arm64
    const ext = platform === "win32" ? ".exe" : "";
    return `borrowscope-lsp-${platform}-${arch}${ext}`;
}
function getBinaryPath(storagePath) {
    return path.join(storagePath, BINARY_NAME);
}
async function ensureServer(context) {
    const storagePath = context.globalStorageUri.fsPath;
    fs.mkdirSync(storagePath, { recursive: true });
    const binaryPath = getBinaryPath(storagePath);
    if (fs.existsSync(binaryPath)) {
        const localVersion = getLocalVersion(binaryPath);
        return { path: binaryPath, version: localVersion };
    }
    // Download with progress
    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: "BorrowScope",
        cancellable: false,
    }, async (progress) => {
        progress.report({ message: "Downloading server..." });
        await downloadServer(storagePath);
        progress.report({ message: "Ready!" });
    });
    return { path: binaryPath, version: getLocalVersion(binaryPath) };
}
function getLocalVersion(binaryPath) {
    if (!fs.existsSync(binaryPath))
        return "unknown";
    // Read version from .version file next to binary
    const versionFile = binaryPath + ".version";
    if (fs.existsSync(versionFile)) {
        return fs.readFileSync(versionFile, "utf8").trim();
    }
    return "unknown";
}
async function getLatestReleaseUrl() {
    const apiUrl = `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`;
    const data = await fetchJson(apiUrl);
    const asset = getPlatformAsset();
    const found = data.assets?.find((a) => a.name === asset);
    if (!found) {
        throw new Error(`No binary found for platform ${process.platform}-${process.arch}`);
    }
    return { url: found.browser_download_url, version: data.tag_name };
}
async function downloadServer(storagePath) {
    const { url, version } = await getLatestReleaseUrl();
    const binaryPath = getBinaryPath(storagePath);
    await downloadFile(url, binaryPath);
    // Make executable on Unix
    if (process.platform !== "win32") {
        fs.chmodSync(binaryPath, 0o755);
    }
    // Write version file
    fs.writeFileSync(binaryPath + ".version", version);
}
async function fetchJson(url) {
    return new Promise((resolve, reject) => {
        const get = url.startsWith("https") ? https.get : http.get;
        get(url, { headers: { "User-Agent": "borrowscope-vscode" } }, (res) => {
            if (res.statusCode === 301 || res.statusCode === 302) {
                fetchJson(res.headers.location).then(resolve, reject);
                return;
            }
            if (res.statusCode !== 200) {
                reject(new Error(`HTTP ${res.statusCode} fetching ${url}`));
                return;
            }
            let body = "";
            res.on("data", (chunk) => (body += chunk));
            res.on("end", () => {
                try {
                    resolve(JSON.parse(body));
                }
                catch (e) {
                    reject(e);
                }
            });
        }).on("error", reject);
    });
}
async function downloadFile(url, dest) {
    return new Promise((resolve, reject) => {
        const get = url.startsWith("https") ? https.get : http.get;
        get(url, { headers: { "User-Agent": "borrowscope-vscode" } }, (res) => {
            if (res.statusCode === 301 || res.statusCode === 302) {
                downloadFile(res.headers.location, dest).then(resolve, reject);
                return;
            }
            if (res.statusCode !== 200) {
                reject(new Error(`Download failed: HTTP ${res.statusCode}`));
                return;
            }
            const file = fs.createWriteStream(dest);
            res.pipe(file);
            file.on("finish", () => { file.close(); resolve(); });
            file.on("error", reject);
        }).on("error", reject);
    });
}
//# sourceMappingURL=server-manager.js.map