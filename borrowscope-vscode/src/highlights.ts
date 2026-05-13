import * as vscode from "vscode";
import { BorrowScope, OwnershipGraph } from "./lifelines";

const sharedHighlight = vscode.window.createTextEditorDecorationType({
  backgroundColor: "rgba(52, 152, 219, 0.06)",
  isWholeLine: true,
});

const mutHighlight = vscode.window.createTextEditorDecorationType({
  backgroundColor: "rgba(231, 76, 60, 0.06)",
  isWholeLine: true,
});

const conflictHighlight = vscode.window.createTextEditorDecorationType({
  backgroundColor: "rgba(241, 196, 15, 0.10)",
  isWholeLine: true,
});

export function applyHighlights(
  editor: vscode.TextEditor,
  scopes: BorrowScope[],
  graph?: OwnershipGraph
): void {
  if (!isHighlightsEnabled()) {
    clearHighlights(editor);
    return;
  }

  const shared: vscode.DecorationOptions[] = [];
  const mutable: vscode.DecorationOptions[] = [];
  const conflicts: vscode.DecorationOptions[] = [];

  for (const s of scopes) {
    const range = new vscode.Range(s.range.start.line, 0, s.range.end.line, Number.MAX_SAFE_INTEGER);
    const dec = {
      range,
      hoverMessage: `${s.is_mutable ? "&mut" : "&"} borrow of \`${s.target}\` by \`${s.borrower}\``,
    };
    if (s.is_mutable) {
      mutable.push(dec);
    } else {
      shared.push(dec);
    }
  }

  if (graph) {
    for (const c of graph.conflicts) {
      const range = new vscode.Range(c.overlap_start_line - 1, 0, c.overlap_end_line - 1, Number.MAX_SAFE_INTEGER);
      conflicts.push({
        range,
        hoverMessage: `Conflict: \`${c.borrow_a}\` and \`${c.borrow_b}\` overlap on \`${c.variable}\``,
      });
    }
  }

  editor.setDecorations(sharedHighlight, shared);
  editor.setDecorations(mutHighlight, mutable);
  editor.setDecorations(conflictHighlight, conflicts);
}

export function clearHighlights(editor: vscode.TextEditor): void {
  editor.setDecorations(sharedHighlight, []);
  editor.setDecorations(mutHighlight, []);
  editor.setDecorations(conflictHighlight, []);
}

export function isHighlightsEnabled(): boolean {
  return vscode.workspace
    .getConfiguration("borrowscope")
    .get<boolean>("decorations.borrowScopes", true);
}

export function disposeHighlights(): void {
  sharedHighlight.dispose();
  mutHighlight.dispose();
  conflictHighlight.dispose();
}
