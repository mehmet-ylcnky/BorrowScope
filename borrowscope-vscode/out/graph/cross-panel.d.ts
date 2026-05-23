import * as vscode from "vscode";
export declare class CrossRefPanel {
    static currentPanel: CrossRefPanel | undefined;
    private readonly _panel;
    private _extensionUri;
    static createOrShow(extensionUri: vscode.Uri, data: any, d3Uri: vscode.Uri): void;
    private constructor();
    private _update;
    private _buildHtml;
}
