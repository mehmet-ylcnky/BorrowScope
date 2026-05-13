import * as vscode from "vscode";

export interface OwnershipHint {
  line: number;
  character: number;
  label: string;
}

// Color-coded decoration types by ownership category
const decorationTypes: Record<string, vscode.TextEditorDecorationType> = {};

function getOrCreateDecorationType(
  label: string
): vscode.TextEditorDecorationType {
  if (decorationTypes[label]) return decorationTypes[label];

  const color = getColorForLabel(label);
  const dt = vscode.window.createTextEditorDecorationType({
    after: {
      contentText: ` ${label}`,
      color,
      fontStyle: "italic",
      margin: "0 0 0 0.5em",
    } as vscode.ThemableDecorationAttachmentRenderOptions,
  });
  decorationTypes[label] = dt;
  return dt;
}

export function getColorForLabel(label: string): string {
  switch (label) {
    case "[&]":
      return "#3498db"; // blue
    case "[&mut]":
      return "#e74c3c"; // red
    case "[Rc]":
      return "#9b59b6"; // purple
    case "[Arc]":
      return "#8e44ad"; // dark purple
    case "[Cell]":
      return "#e67e22"; // orange
    case "[*ptr]":
      return "#7f8c8d"; // gray
    case "[closure]":
      return "#27ae60"; // green
    default:
      return "#95a5a6"; // light gray
  }
}

export function applyDecorations(
  editor: vscode.TextEditor,
  hints: OwnershipHint[]
): void {
  if (!isEnabled()) {
    clearDecorations(editor);
    return;
  }

  // Group hints by label
  const grouped: Record<string, vscode.DecorationOptions[]> = {};
  for (const hint of hints) {
    if (!grouped[hint.label]) grouped[hint.label] = [];
    const pos = new vscode.Position(hint.line, hint.character);
    grouped[hint.label].push({ range: new vscode.Range(pos, pos) });
  }

  // Clear all existing decorations first
  for (const dt of Object.values(decorationTypes)) {
    editor.setDecorations(dt, []);
  }

  // Apply grouped decorations
  for (const [label, ranges] of Object.entries(grouped)) {
    const dt = getOrCreateDecorationType(label);
    editor.setDecorations(dt, ranges);
  }
}

export function clearDecorations(editor: vscode.TextEditor): void {
  for (const dt of Object.values(decorationTypes)) {
    editor.setDecorations(dt, []);
  }
}

export function isEnabled(): boolean {
  return vscode.workspace
    .getConfiguration("borrowscope")
    .get<boolean>("decorations.enabled", true);
}

export function disposeDecorations(): void {
  for (const dt of Object.values(decorationTypes)) {
    dt.dispose();
  }
  for (const key of Object.keys(decorationTypes)) {
    delete decorationTypes[key];
  }
}
