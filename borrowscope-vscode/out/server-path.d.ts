export interface ServerPathContext {
    extensionPath: string;
    configuredPath: string;
    globalStoragePath?: string;
}
export declare function resolveServerPath(ctx: ServerPathContext): string;
