import * as vscode from "vscode";

export interface BorrowScope {
  borrower: string;
  target: string;
  is_mutable: boolean;
  range: { start: { line: number }; end: { line: number } };
}

interface Lifeline {
  startLine: number;
  endLine: number;
  color: string;
  label: string;
  isMutable: boolean;
}

const COLORS = {
  shared: "#3498db",    // blue
  mutable: "#e74c3c",   // red
  owner: "#2ecc71",     // green
  rc: "#9b59b6",        // purple
  conflict: "#f1c40f",  // yellow
};

// Decoration types for flow characters (one per color)
const flowDecorationTypes: Map<string, vscode.TextEditorDecorationType> = new Map();

function getFlowDecorationType(color: string): vscode.TextEditorDecorationType {
  if (flowDecorationTypes.has(color)) return flowDecorationTypes.get(color)!;
  const dt = vscode.window.createTextEditorDecorationType({
    before: {
      color,
      margin: "0 0.5em 0 0",
      fontWeight: "bold",
    } as vscode.ThemableDecorationAttachmentRenderOptions,
  });
  flowDecorationTypes.set(color, dt);
  return dt;
}

export function applyLifelines(
  editor: vscode.TextEditor,
  scopes: BorrowScope[]
): void {
  if (!isLifelinesEnabled()) {
    clearLifelines(editor);
    return;
  }

  // Convert scopes to lifelines
  const lifelines: Lifeline[] = scopes.map((s) => ({
    startLine: s.range.start.line,
    endLine: s.range.end.line,
    color: s.is_mutable ? COLORS.mutable : COLORS.shared,
    label: `${s.borrower} borrows ${s.target}`,
    isMutable: s.is_mutable,
  }));

  // Build per-line flow characters
  // Group decorations by color
  const decorationsByColor: Map<string, vscode.DecorationOptions[]> = new Map();

  for (const ll of lifelines) {
    const color = ll.color;
    if (!decorationsByColor.has(color)) decorationsByColor.set(color, []);
    const decs = decorationsByColor.get(color)!;

    for (let line = ll.startLine; line <= ll.endLine; line++) {
      let char: string;
      if (line === ll.startLine) {
        char = "├─";  // branch start
      } else if (line === ll.endLine) {
        char = "╰─";  // branch end
      } else {
        char = "│ ";   // continuation
      }

      decs.push({
        range: new vscode.Range(line, 0, line, 0),
        renderOptions: {
          before: {
            contentText: char,
            color,
            fontWeight: "bold",
          },
        },
        hoverMessage: line === ll.startLine
          ? `Borrow starts: ${ll.label} (${ll.isMutable ? "&mut" : "&"})`
          : line === ll.endLine
          ? `Borrow ends: ${ll.label}`
          : `${ll.isMutable ? "&mut" : "&"} borrow active`,
      });
    }
  }

  // Clear all existing
  clearLifelines(editor);

  // Apply per-color decorations
  for (const [color, decs] of decorationsByColor) {
    const dt = getFlowDecorationType(color);
    editor.setDecorations(dt, decs);
  }
}

export function clearLifelines(editor: vscode.TextEditor): void {
  for (const dt of flowDecorationTypes.values()) {
    editor.setDecorations(dt, []);
  }
}

export function isLifelinesEnabled(): boolean {
  return vscode.workspace
    .getConfiguration("borrowscope")
    .get<boolean>("decorations.enabled", true);
}

export function disposeLifelines(): void {
  for (const dt of flowDecorationTypes.values()) {
    dt.dispose();
  }
  flowDecorationTypes.clear();
}
