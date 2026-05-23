"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.computeOwnershipDiff = computeOwnershipDiff;
function computeOwnershipDiff(before, after) {
    const beforeVars = new Set((before?.variables || []).map((v) => v.name));
    const afterVars = new Set((after?.variables || []).map((v) => v.name));
    const beforeBorrows = new Set((before?.borrow_scopes || []).map((s) => `${s.borrower_name}->${s.target_name}`));
    const afterBorrows = new Set((after?.borrow_scopes || []).map((s) => `${s.borrower_name}->${s.target_name}`));
    const beforeMoves = new Set((before?.moves || []).map((m) => m.source_name));
    const afterMoves = new Set((after?.moves || []).map((m) => m.source_name));
    const addedVariables = [...afterVars].filter(v => !beforeVars.has(v));
    const removedVariables = [...beforeVars].filter(v => !afterVars.has(v));
    const addedBorrows = [...afterBorrows].filter(b => !beforeBorrows.has(b));
    const removedBorrows = [...beforeBorrows].filter(b => !afterBorrows.has(b));
    const addedMoves = [...afterMoves].filter(m => !beforeMoves.has(m));
    const removedMoves = [...beforeMoves].filter(m => !afterMoves.has(m));
    const parts = [];
    if (addedVariables.length)
        parts.push(`+${addedVariables.length} vars`);
    if (removedVariables.length)
        parts.push(`-${removedVariables.length} vars`);
    if (addedBorrows.length)
        parts.push(`+${addedBorrows.length} borrows`);
    if (removedBorrows.length)
        parts.push(`-${removedBorrows.length} borrows`);
    if (addedMoves.length)
        parts.push(`+${addedMoves.length} moves`);
    if (removedMoves.length)
        parts.push(`-${removedMoves.length} moves`);
    const hasChanges = parts.length > 0;
    const summary = hasChanges ? parts.join(", ") : "No ownership changes";
    return { addedVariables, removedVariables, addedBorrows, removedBorrows, addedMoves, removedMoves, summary, hasChanges };
}
//# sourceMappingURL=comparison.js.map