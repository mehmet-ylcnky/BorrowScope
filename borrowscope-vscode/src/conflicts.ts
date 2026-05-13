import * as vscode from "vscode";

const conflictDecoration = vscode.window.createTextEditorDecorationType({
  textDecoration: "underline wavy rgba(241, 196, 15, 0.8)",
  overviewRulerColor: "rgba(241, 196, 15, 0.8)",
  overviewRulerLane: vscode.OverviewRulerLane.Center,
});

export interface ConflictDiagnostic {
  range: { start: { line: number; character: number }; end: { line: number; character: number } };
  message: string;
  source?: string;
}

export function applyConflictDecorations(
  editor: vscode.TextEditor,
  diagnostics: ConflictDiagnostic[]
): void {
  const borrowScopeDiags = diagnostics.filter(
    (d) => d.source === "BorrowScope"
  );

  if (borrowScopeDiags.length === 0) {
    clearConflictDecorations(editor);
    return;
  }

  const decorations: vscode.DecorationOptions[] = borrowScopeDiags.map((d) => ({
    range: new vscode.Range(
      d.range.start.line,
      d.range.start.character,
      d.range.end.line,
      d.range.end.character
    ),
    hoverMessage: `⚠️ ${d.message}`,
  }));

  editor.setDecorations(conflictDecoration, decorations);
}

export function clearConflictDecorations(editor: vscode.TextEditor): void {
  editor.setDecorations(conflictDecoration, []);
}

export function disposeConflictDecorations(): void {
  conflictDecoration.dispose();
}
