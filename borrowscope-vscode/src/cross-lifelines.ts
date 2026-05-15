import * as vscode from "vscode";

export interface CrossBorrow {
  origin_variable: string;
  origin_line: number;
  path: CrossPathSegment[];
}

export interface CrossPathSegment {
  file: string;
  function_name: string;
  variable: string;
  start_line: number;
  end_line: number;
  is_mutable: boolean;
  kind: string;
}

const crossLifelineDecoration = vscode.window.createTextEditorDecorationType({
  isWholeLine: true,
});

export function applyCrossLifelines(
  editor: vscode.TextEditor,
  crossBorrows: CrossBorrow[]
): void {
  const decorations: vscode.DecorationOptions[] = [];
  const cursorLine = editor.selection.active.line;

  // Only show connections for the function the cursor is in
  let cursorFn = "";
  for (let i = cursorLine; i >= 0; i--) {
    const match = editor.document.lineAt(i).text.match(/\bfn\s+(\w+)/);
    if (match) { cursorFn = match[1]; break; }
  }

  for (const cb of crossBorrows) {
    if (cb.path.length < 2) continue;

    const origin = cb.path[0];
    const target = cb.path[1];

    // Only render if this borrow originates from the cursor's function
    if (origin.function_name !== cursorFn) continue;

    const isMut = target.is_mutable;
    const color = isMut ? "rgba(231, 76, 60, 0.5)" : "rgba(26, 188, 156, 0.5)";

    // Find the callee function's line in the document
    const calleeLine = findFunctionLine(editor, target.function_name);
    if (calleeLine < 0) continue;

    const originLine = (origin.start_line || 1) - 1;

    // Draw dashed connection between caller and callee
    // Lines between the call and the callee function declaration
    const startLine = Math.min(originLine, calleeLine);
    const endLine = Math.max(originLine, calleeLine);

    for (let line = startLine + 1; line < endLine; line++) {
      decorations.push({
        range: new vscode.Range(line, 0, line, 0),
        renderOptions: {
          before: {
            contentText: "┊ ",
            color,
          },
        },
      });
    }

    // Annotation on callee's function line: "◄── received from caller"
    decorations.push({
      range: new vscode.Range(calleeLine, 0, calleeLine, 0),
      renderOptions: {
        after: {
          contentText: ` ◄── ${isMut ? "🔒" : "👁"} receives &${cb.origin_variable} from ${origin.function_name}()`,
          color,
          fontStyle: "italic",
          margin: "0 0 0 2em",
        } as vscode.ThemableDecorationAttachmentRenderOptions,
      },
      hoverMessage: `**Cross-function borrow**\n\n\`${target.variable}\` in \`${target.function_name}()\` receives \`&${cb.origin_variable}\` from \`${origin.function_name}()\``,
    });
  }

  editor.setDecorations(crossLifelineDecoration, decorations);
}

export function clearCrossLifelines(editor: vscode.TextEditor): void {
  editor.setDecorations(crossLifelineDecoration, []);
}

function findFunctionLine(editor: vscode.TextEditor, fnName: string): number {
  for (let i = 0; i < editor.document.lineCount; i++) {
    if (new RegExp(`\\bfn\\s+${fnName}\\b`).test(editor.document.lineAt(i).text)) {
      return i;
    }
  }
  return -1;
}
