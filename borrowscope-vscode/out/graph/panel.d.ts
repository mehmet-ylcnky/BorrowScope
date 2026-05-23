import * as vscode from "vscode";
export declare class GraphPanel {
    static currentPanel: GraphPanel | undefined;
    private static _context;
    private readonly _panel;
    private _disposables;
    private _extensionUri;
    private _currentGraph;
    private _previousGraph;
    private static _lastView;
    private static readonly STATE_KEY;
    static setContext(context: vscode.ExtensionContext): void;
    static createOrShow(extensionUri: vscode.Uri, graph?: any, functionList?: string[]): void;
    static getPanel(): GraphPanel | undefined;
    private constructor();
    private _loadFunction;
    updateGraph(graph: any, functionList?: string[]): void;
    getGraph(): any | undefined;
    getPanel(): vscode.WebviewPanel;
    dispose(): void;
    private _buildHtml;
    category: any;
    $: any;
}
