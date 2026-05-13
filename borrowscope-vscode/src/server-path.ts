import * as path from "path";
import * as fs from "fs";

export interface ServerPathContext {
  extensionPath: string;
  configuredPath: string;
}

export function resolveServerPath(ctx: ServerPathContext): string {
  // 1. User-configured path
  if (ctx.configuredPath && fs.existsSync(ctx.configuredPath)) {
    return ctx.configuredPath;
  }

  // 2. Bundled binary
  const bundled = path.join(ctx.extensionPath, "server", "borrowscope-lsp");
  if (fs.existsSync(bundled)) {
    return bundled;
  }

  // 3. System PATH
  const pathDirs = (process.env.PATH || "").split(path.delimiter);
  for (const dir of pathDirs) {
    const candidate = path.join(dir, "borrowscope-lsp");
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  throw new Error(
    "borrowscope-lsp binary not found. Install it or set borrowscope.server.path."
  );
}
