import * as vscode from "vscode";
export interface PrerequisiteStatus {
    rustToolchain: boolean;
    rustVersion: string;
    cargoProject: boolean;
    projectName: string;
    serverBinary: boolean;
}
/** Show welcome panel on first activation */
export declare function showWelcomeIfNeeded(context: vscode.ExtensionContext): void;
/** Show the welcome panel (can be triggered manually) */
export declare function showWelcomePanel(context: vscode.ExtensionContext): void;
/** Check if prerequisites are met */
export declare function checkPrerequisites(): Promise<PrerequisiteStatus>;
