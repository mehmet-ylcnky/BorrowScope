"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildTimelineData = buildTimelineData;
function buildTimelineData(graph) {
    const variables = (graph.variables || []).map((v) => ({
        name: v.name,
        type: v.type_display,
        category: v.ownership_category,
        startLine: v.line,
        endLine: v.line + 5, // default span if no end info
    }));
    const borrowScopes = (graph.borrow_scopes || []).map((s) => ({
        borrower: s.borrower_name,
        target: s.target_name,
        isMutable: s.is_mutable,
        startLine: s.start_line,
        endLine: s.end_line,
    }));
    // Update variable end lines from borrow scopes and function end
    for (const v of variables) {
        // Find last borrow that references this variable
        const lastBorrow = borrowScopes
            .filter(b => b.target === v.name || b.borrower === v.name)
            .reduce((max, b) => Math.max(max, b.endLine), v.startLine);
        v.endLine = Math.max(v.endLine, lastBorrow, graph.end_line || v.startLine + 5);
    }
    // Update borrower variable end lines to match their scope
    for (const b of borrowScopes) {
        const borrowerVar = variables.find(v => v.name === b.borrower);
        if (borrowerVar) {
            borrowerVar.startLine = Math.min(borrowerVar.startLine, b.startLine);
            borrowerVar.endLine = Math.max(borrowerVar.endLine, b.endLine);
        }
    }
    const conflicts = (graph.conflicts || []).map((c) => ({
        variable: c.variable,
        borrowA: c.borrow_a,
        borrowB: c.borrow_b,
        startLine: c.overlap_start_line,
        endLine: c.overlap_end_line,
    }));
    const allLines = variables.flatMap(v => [v.startLine, v.endLine]);
    const minLine = allLines.length > 0 ? Math.min(...allLines) : 1;
    const maxLine = allLines.length > 0 ? Math.max(...allLines) : 10;
    return {
        functionName: graph.function_name || "",
        minLine,
        maxLine,
        variables,
        borrowScopes,
        conflicts,
    };
}
//# sourceMappingURL=timeline.js.map