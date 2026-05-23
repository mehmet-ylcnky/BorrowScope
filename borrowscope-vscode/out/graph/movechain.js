"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildMoveChainData = buildMoveChainData;
function buildMoveChainData(graph) {
    const moves = graph.moves || [];
    const variables = graph.variables || [];
    if (moves.length === 0) {
        return { chains: [], hasMovesFlag: false };
    }
    // Build adjacency: source -> destination
    const edges = moves.map((m) => ({
        from: m.source_name,
        to: extractDest(m.destination),
        line: m.line,
    }));
    // Find all nodes involved in moves
    const nodeNames = new Set();
    for (const e of edges) {
        nodeNames.add(e.from);
        nodeNames.add(e.to);
    }
    // Determine which are alive (not moved away)
    const movedAway = new Set(edges.map(e => e.from));
    const allNodes = [...nodeNames].map(name => {
        const v = variables.find((vv) => vv.name === name);
        return {
            name,
            type: v ? v.type_display : "unknown",
            line: v ? v.line : 0,
            isAlive: !movedAway.has(name),
        };
    });
    // Group into chains (connected components)
    const visited = new Set();
    const chains = [];
    for (const node of allNodes) {
        if (visited.has(node.name))
            continue;
        const chain = buildChain(node.name, edges, allNodes, visited);
        if (chain.nodes.length > 0)
            chains.push(chain);
    }
    return { chains, hasMovesFlag: true };
}
function buildChain(start, edges, allNodes, visited) {
    const chainNodes = [];
    const chainEdges = [];
    const queue = [start];
    while (queue.length > 0) {
        const name = queue.shift();
        if (visited.has(name))
            continue;
        visited.add(name);
        const node = allNodes.find(n => n.name === name);
        if (node)
            chainNodes.push(node);
        for (const e of edges) {
            if (e.from === name) {
                chainEdges.push(e);
                if (!visited.has(e.to))
                    queue.push(e.to);
            }
            if (e.to === name) {
                if (!visited.has(e.from))
                    queue.push(e.from);
            }
        }
    }
    return { nodes: chainNodes, edges: chainEdges };
}
function extractDest(destination) {
    if (typeof destination === "string")
        return destination;
    if (destination && typeof destination === "object") {
        if (destination.Variable)
            return destination.Variable;
        if (destination.FunctionArg)
            return destination.FunctionArg.function + "()";
        if (destination.Return)
            return "(return)";
        if (destination.Closure)
            return "(closure)";
    }
    return String(destination).slice(0, 15);
}
//# sourceMappingURL=movechain.js.map