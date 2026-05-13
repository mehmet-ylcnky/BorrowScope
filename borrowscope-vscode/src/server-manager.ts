import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import * as https from "https";
import * as http from "http";

const GITHUB_REPO = "mehmet-ylcnky/BorrowScope";
const BINARY_NAME =
  process.platform === "win32" ? "borrowscope-lsp.exe" : "borrowscope-lsp";

export interface ServerBinary {
  path: string;
  version: string;
}

export function getPlatformAsset(): string {
  const platform = process.platform; // linux, darwin, win32
  const arch = process.arch; // x64, arm64
  const ext = platform === "win32" ? ".exe" : "";
  return `borrowscope-lsp-${platform}-${arch}${ext}`;
}

export function getBinaryPath(storagePath: string): string {
  return path.join(storagePath, BINARY_NAME);
}

export async function ensureServer(
  context: vscode.ExtensionContext
): Promise<ServerBinary> {
  const storagePath = context.globalStorageUri.fsPath;
  fs.mkdirSync(storagePath, { recursive: true });

  const binaryPath = getBinaryPath(storagePath);

  if (fs.existsSync(binaryPath)) {
    const localVersion = getLocalVersion(binaryPath);
    return { path: binaryPath, version: localVersion };
  }

  // Download with progress
  await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: "BorrowScope",
      cancellable: false,
    },
    async (progress) => {
      progress.report({ message: "Downloading server..." });
      await downloadServer(storagePath);
      progress.report({ message: "Ready!" });
    }
  );

  return { path: binaryPath, version: getLocalVersion(binaryPath) };
}

export function getLocalVersion(binaryPath: string): string {
  if (!fs.existsSync(binaryPath)) return "unknown";
  // Read version from .version file next to binary
  const versionFile = binaryPath + ".version";
  if (fs.existsSync(versionFile)) {
    return fs.readFileSync(versionFile, "utf8").trim();
  }
  return "unknown";
}

export async function getLatestReleaseUrl(): Promise<{
  url: string;
  version: string;
}> {
  const apiUrl = `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`;
  const data = await fetchJson(apiUrl);
  const asset = getPlatformAsset();
  const found = data.assets?.find((a: any) => a.name === asset);
  if (!found) {
    throw new Error(
      `No binary found for platform ${process.platform}-${process.arch}`
    );
  }
  return { url: found.browser_download_url, version: data.tag_name };
}

export async function downloadServer(storagePath: string): Promise<void> {
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

async function fetchJson(url: string): Promise<any> {
  return new Promise((resolve, reject) => {
    const get = url.startsWith("https") ? https.get : http.get;
    get(
      url,
      { headers: { "User-Agent": "borrowscope-vscode" } },
      (res) => {
        if (res.statusCode === 301 || res.statusCode === 302) {
          fetchJson(res.headers.location!).then(resolve, reject);
          return;
        }
        if (res.statusCode !== 200) {
          reject(new Error(`HTTP ${res.statusCode} fetching ${url}`));
          return;
        }
        let body = "";
        res.on("data", (chunk) => (body += chunk));
        res.on("end", () => {
          try { resolve(JSON.parse(body)); }
          catch (e) { reject(e); }
        });
      }
    ).on("error", reject);
  });
}

async function downloadFile(url: string, dest: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const get = url.startsWith("https") ? https.get : http.get;
    get(
      url,
      { headers: { "User-Agent": "borrowscope-vscode" } },
      (res) => {
        if (res.statusCode === 301 || res.statusCode === 302) {
          downloadFile(res.headers.location!, dest).then(resolve, reject);
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
      }
    ).on("error", reject);
  });
}
