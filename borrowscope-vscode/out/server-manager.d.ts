import * as vscode from "vscode";
export interface ServerBinary {
    path: string;
    version: string;
}
export declare function getPlatformAsset(): string;
export declare function getBinaryPath(storagePath: string): string;
export declare function ensureServer(context: vscode.ExtensionContext): Promise<ServerBinary>;
export declare function getLocalVersion(binaryPath: string): string;
export declare function getLatestReleaseUrl(): Promise<{
    url: string;
    version: string;
}>;
export declare function downloadServer(storagePath: string): Promise<void>;
