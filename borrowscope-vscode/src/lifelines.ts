import * as vscode from "vscode";

export interface BorrowScope {
  borrower: string;
  target: string;
  is_mutable: boolean;
  range: { start: { line: number }; end: { line: number } };
}

export interface OwnershipGraph {
  function_name: string;
  start_line: number;
  end_line: number;
  variables: Array<{ name: string; line: number; ownership_category: string }>;
  borrow_scopes: BorrowScope[];
  moves: Array<{ source_name: string; line: number; destination: any; source_type: string }>;
  rc_clones: Array<{ clone_variable: string; source_variable: string; clone_type: string; line: number }>;
  conflicts: Array<{ variable: string; borrow_a: string; borrow_b: string; overlap_start_line: number; overlap_end_line: number }>;
}

const COLORS = {
  owner: "#2ecc71",     // green
  shared: "#3498db",    // blue
  mutable: "#e74c3c",   // red
  rc: "#9b59b6",        // purple
  conflict: "#f1c40f",  // yellow
  move: "#e67e22",      // orange
};

// One decoration type per color — supports renderOptions per-item
const lifelineDecorationTypes: Map<string, vscode.TextEditorDecorationType> = new Map();

function getDecorationType(color: string): vscode.TextEditorDecorationType {
  if (lifelineDecorationTypes.has(color)) return lifelineDecorationTypes.get(color)!;
  const dt = vscode.window.createTextEditorDecorationType({
    isWholeLine: true,
  });
  lifelineDecorationTypes.set(color, dt);
  return dt;
}

interface LineDecoration {
  line: number;
  char: string;
  suffix: string;
  color: string;
  hover: string;
}

export function buildLifelineDecorations(
  scopes: BorrowScope[],
  graph?: OwnershipGraph
): LineDecoration[] {
  const decorations: LineDecoration[] = [];

  // 1. Owner lifelines — DISABLED (too noisy, owners span entire function)
  // Only show lifelines for borrows, moves, clones, and conflicts

  // 2. Shared borrow lifelines (blue) + frozen indicator on target
  for (const s of scopes.filter((s) => !s.is_mutable)) {
    for (let line = s.range.start.line; line <= s.range.end.line; line++) {
      let char = "│ ";
      let suffix = "";
      if (line === s.range.start.line) {
        char = "├─";
        suffix = ` 👁 &${s.borrower} ⟵ ${s.target}`;
      } else if (line === s.range.end.line) {
        char = "╰─";
        suffix = ` 💧 ${s.borrower} released`;
      }
      decorations.push({
        line, char, suffix, color: COLORS.shared,
        hover: `& borrow: ${s.borrower} reads ${s.target}`,
      });
    }
    // Frozen indicator: show ❄ on the target variable's line during borrow
    decorations.push({
      line: s.range.start.line, char: "❄ ", suffix: `${s.target} frozen (immutably borrowed)`,
      color: "rgba(52, 152, 219, 0.4)",
      hover: `${s.target} is frozen: cannot move or mutably borrow while ${s.borrower} exists`,
    });
  }

  // 3. Mutable borrow lifelines (red) + locked indicator on target
  for (const s of scopes.filter((s) => s.is_mutable)) {
    for (let line = s.range.start.line; line <= s.range.end.line; line++) {
      let char = "│ ";
      let suffix = "";
      if (line === s.range.start.line) {
        char = "├─";
        suffix = ` 🔒 &mut ${s.borrower} ⟵ ${s.target}`;
      } else if (line === s.range.end.line) {
        char = "╰─";
        suffix = ` 💧 ${s.borrower} released`;
      }
      decorations.push({
        line, char, suffix, color: COLORS.mutable,
        hover: `&mut borrow: ${s.borrower} exclusively locks ${s.target}`,
      });
    }
    // Locked indicator: show 🔒 on target during mutable borrow
    decorations.push({
      line: s.range.start.line, char: "🔒", suffix: `${s.target} locked (exclusively borrowed)`,
      color: "rgba(231, 76, 60, 0.4)",
      hover: `${s.target} is locked: cannot read, write, or borrow while ${s.borrower} exists`,
    });
  }

  // 4. Rc/Arc clone events (purple, single line marker)
  if (graph) {
    for (const rc of graph.rc_clones) {
      const line = rc.line - 1;
      decorations.push({
        line, char: "├─", suffix: ` 🔗 ${rc.clone_variable} cloned from ${rc.source_variable}`,
        color: COLORS.rc,
        hover: `${rc.clone_type}: ${rc.clone_variable} shares ownership with ${rc.source_variable}`,
      });
    }
  }

  // 5. Conflict zones (yellow)
  if (graph) {
    for (const c of graph.conflicts) {
      for (let line = c.overlap_start_line - 1; line <= c.overlap_end_line - 1; line++) {
        let suffix = "";
        if (line === c.overlap_start_line - 1) {
          suffix = ` ⚠️ ${c.borrow_a} and ${c.borrow_b} overlap on ${c.variable}`;
        }
        decorations.push({
          line, char: "┃ ", suffix, color: COLORS.conflict,
          hover: `Conflict: ${c.borrow_a} and ${c.borrow_b} both borrow ${c.variable}`,
        });
      }
    }
  }

  // 6. Move events (orange) + drop bar showing variable is dead
  if (graph) {
    for (const m of graph.moves) {
      const line = m.line - 1;
      const dest = typeof m.destination === "string" ? m.destination : JSON.stringify(m.destination);
      decorations.push({
        line, char: "↦ ", suffix: ` ${m.source_name} ⤳ ${dest}`,
        color: COLORS.move,
        hover: `Move: ${m.source_name} (${m.source_type}) ownership transferred to ${dest}`,
      });
      // Dead indicator: variable can no longer be used after this line
      decorations.push({
        line, char: "─┘", suffix: `${m.source_name} dead (moved away)`,
        color: "rgba(231, 76, 60, 0.3)",
        hover: `${m.source_name} is no longer accessible after this line`,
      });
    }
  }

  return decorations;
}

export function applyLifelines(
  editor: vscode.TextEditor,
  scopes: BorrowScope[],
  graph?: OwnershipGraph
): void {
  if (!isLifelinesEnabled()) {
    clearLifelines(editor);
    return;
  }

  const decorations = buildLifelineDecorations(scopes, graph);

  // Group by color
  const grouped: Map<string, vscode.DecorationOptions[]> = new Map();
  for (const d of decorations) {
    if (!grouped.has(d.color)) grouped.set(d.color, []);
    grouped.get(d.color)!.push({
      range: new vscode.Range(d.line, 0, d.line, 0),
      renderOptions: {
        before: { contentText: d.char, color: d.color, fontWeight: "bold" },
        after: d.suffix ? {
          contentText: d.suffix,
          color: "rgba(150,150,150,0.7)",
          fontStyle: "italic",
          margin: "0 0 0 2em",
        } as vscode.ThemableDecorationAttachmentRenderOptions : undefined,
      },
      hoverMessage: d.hover,
    });
  }

  // Clear existing
  clearLifelines(editor);

  // Apply
  for (const [color, decs] of grouped) {
    const dt = getDecorationType(color);
    editor.setDecorations(dt, decs);
  }
}

export function clearLifelines(editor: vscode.TextEditor): void {
  for (const dt of lifelineDecorationTypes.values()) {
    editor.setDecorations(dt, []);
  }
}

export function isLifelinesEnabled(): boolean {
  return vscode.workspace
    .getConfiguration("borrowscope")
    .get<boolean>("decorations.enabled", true);
}

export function disposeLifelines(): void {
  for (const dt of lifelineDecorationTypes.values()) {
    dt.dispose();
  }
  lifelineDecorationTypes.clear();
}
