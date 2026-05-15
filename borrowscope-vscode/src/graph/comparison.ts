export interface OwnershipDiff {
  addedVariables: string[];
  removedVariables: string[];
  addedBorrows: string[];
  removedBorrows: string[];
  addedMoves: string[];
  removedMoves: string[];
  summary: string;
  hasChanges: boolean;
}

export function computeOwnershipDiff(before: any, after: any): OwnershipDiff {
  const beforeVars = new Set((before?.variables || []).map((v: any) => v.name));
  const afterVars = new Set((after?.variables || []).map((v: any) => v.name));

  const beforeBorrows = new Set((before?.borrow_scopes || []).map((s: any) => `${s.borrower_name}->${s.target_name}`));
  const afterBorrows = new Set((after?.borrow_scopes || []).map((s: any) => `${s.borrower_name}->${s.target_name}`));

  const beforeMoves = new Set((before?.moves || []).map((m: any) => m.source_name));
  const afterMoves = new Set((after?.moves || []).map((m: any) => m.source_name));

  const addedVariables: string[] = [...afterVars].filter(v => !beforeVars.has(v)) as string[];
  const removedVariables: string[] = [...beforeVars].filter(v => !afterVars.has(v)) as string[];
  const addedBorrows: string[] = [...afterBorrows].filter(b => !beforeBorrows.has(b)) as string[];
  const removedBorrows: string[] = [...beforeBorrows].filter(b => !afterBorrows.has(b)) as string[];
  const addedMoves: string[] = [...afterMoves].filter(m => !beforeMoves.has(m)) as string[];
  const removedMoves: string[] = [...beforeMoves].filter(m => !afterMoves.has(m)) as string[];

  const parts: string[] = [];
  if (addedVariables.length) parts.push(`+${addedVariables.length} vars`);
  if (removedVariables.length) parts.push(`-${removedVariables.length} vars`);
  if (addedBorrows.length) parts.push(`+${addedBorrows.length} borrows`);
  if (removedBorrows.length) parts.push(`-${removedBorrows.length} borrows`);
  if (addedMoves.length) parts.push(`+${addedMoves.length} moves`);
  if (removedMoves.length) parts.push(`-${removedMoves.length} moves`);

  const hasChanges = parts.length > 0;
  const summary = hasChanges ? parts.join(", ") : "No ownership changes";

  return { addedVariables, removedVariables, addedBorrows, removedBorrows, addedMoves, removedMoves, summary, hasChanges };
}
