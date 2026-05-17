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
  const cfg = vscode.workspace.getConfiguration("borrowscope.colors");
  switch (label) {
    case "[&]":
      return cfg.get("sharedBorrow", "#3498db");
    case "[&mut]":
      return cfg.get("mutableBorrow", "#e74c3c");
    case "[Rc]":
      return cfg.get("rcArc", "#9b59b6");
    case "[Arc]":
      return cfg.get("rcArc", "#9b59b6");
    case "[Cell]":
      return cfg.get("move", "#e67e22");
    case "[*ptr]":
      return cfg.get("drop", "#95a5a6");
    case "[closure]":
      return cfg.get("owned", "#2ecc71");
    default:
      return cfg.get("drop", "#95a5a6");
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
