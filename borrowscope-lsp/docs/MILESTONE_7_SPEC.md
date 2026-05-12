# Milestone 7: Advanced Visualizations - Detailed Specification

## 7.1 Temporal View: Borrow Lifetime Timeline

**Objective:** Display a Gantt-chart style timeline showing when each variable is alive and when each borrow is active. The horizontal axis is source line number (not runtime timestamp, since this is static analysis). Each variable gets a horizontal bar from its declaration line to its drop/last-use line.

**Visual design:**
```
Line:  5    10    15    20    25    30    35    40
       │     │     │     │     │     │     │     │
data   ████████████████████████████████████████████  (lines 5-40)
r      ·····▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓·····················  (borrow: lines 8-22)
m      ·····························▓▓▓▓▓▓▓▓▓▓▓▓···  (borrow: lines 25-35)
rc     ████████████████████████████████████████████  (lines 6-40)
rc2    ·········████████████████████████·············  (lines 10-30)

Legend: ████ = alive   ▓▓▓▓ = active borrow   ····· = not yet created / dropped
        Red overlay = conflict zone (if borrows overlap with &mut)
```

**Steps:**
1. Collect all variables with their declaration line and drop/last-use line
2. Collect all borrow scopes with start/end lines
3. Render as horizontal bars in a scrollable SVG
4. Highlight conflict zones (overlapping &mut with any other borrow) in red
5. Click a bar to navigate to that line in the editor

**Code (media/timeline.js):**
```javascript
function renderTimeline(data) {
    const { variables, borrowScopes, conflicts } = data;
    const margin = { top: 30, right: 20, bottom: 30, left: 100 };
    const barHeight = 24;
    const height = variables.length * (barHeight + 4) + margin.top + margin.bottom;

    const minLine = Math.min(...variables.map(v => v.startLine));
    const maxLine = Math.max(...variables.map(v => v.endLine || v.startLine + 10));

    const xScale = d3.scaleLinear()
        .domain([minLine, maxLine])
        .range([margin.left, width - margin.right]);

    const svg = d3.select('#timeline-container').append('svg')
        .attr('width', '100%')
        .attr('height', height);

    // Variable lifetime bars
    variables.forEach((v, i) => {
        const y = margin.top + i * (barHeight + 4);

        // Label
        svg.append('text')
            .attr('x', margin.left - 10)
            .attr('y', y + barHeight / 2 + 4)
            .attr('text-anchor', 'end')
            .attr('font-size', '11px')
            .attr('fill', 'var(--vscode-foreground)')
            .text(v.name);

        // Lifetime bar
        svg.append('rect')
            .attr('x', xScale(v.startLine))
            .attr('y', y)
            .attr('width', xScale(v.endLine || maxLine) - xScale(v.startLine))
            .attr('height', barHeight)
            .attr('rx', 3)
            .attr('fill', categoryColor(v.category))
            .attr('opacity', 0.4)
            .on('click', () => navigateToLine(v.startLine));
    });

    // Borrow scope overlays
    borrowScopes.forEach(scope => {
        const varIndex = variables.findIndex(v => v.name === scope.borrower);
        if (varIndex < 0) return;
        const y = margin.top + varIndex * (barHeight + 4);

        svg.append('rect')
            .attr('x', xScale(scope.startLine))
            .attr('y', y + 2)
            .attr('width', xScale(scope.endLine) - xScale(scope.startLine))
            .attr('height', barHeight - 4)
            .attr('rx', 2)
            .attr('fill', scope.isMutable ? '#e74c3c' : '#3498db')
            .attr('opacity', 0.7);
    });

    // Conflict zones (red vertical bands)
    conflicts.forEach(c => {
        svg.append('rect')
            .attr('x', xScale(c.startLine))
            .attr('y', margin.top - 5)
            .attr('width', xScale(c.endLine) - xScale(c.startLine))
            .attr('height', height - margin.top - margin.bottom + 10)
            .attr('fill', 'rgba(231, 76, 60, 0.1)')
            .attr('stroke', '#e74c3c')
            .attr('stroke-dasharray', '4,2');
    });

    // Line number axis
    svg.append('g')
        .attr('transform', `translate(0, ${height - margin.bottom})`)
        .call(d3.axisBottom(xScale).ticks(10).tickFormat(d => `L${d}`));
}
```

**LSP request:**
```
method: "borrowscope/timeline"
params: { textDocument: { uri }, position }
response: { variables: [...], borrowScopes: [...], conflicts: [...] }
```

**Expectation:** The timeline provides an instant visual answer to "when is each variable alive?" and "do any borrows overlap?" without reading the code line by line.

**Tests for 7.1:**
- Variables render as horizontal bars at correct line positions
- Borrow scopes overlay on the correct variable's bar
- Mutable borrows are red, shared borrows are blue
- Conflict zones appear as red vertical bands
- Clicking a bar navigates to the correct line
- Timeline scrolls horizontally for long functions
- Empty function shows empty timeline (no crash)

---

## 7.2 Scope Nesting View

**Objective:** Display variables grouped by their containing scope (function, block, loop, match arm). Nested scopes appear as nested boxes, making it visually clear which variables are dropped together and in what order.

**Visual design:**
```
┌─ fn process_data() ──────────────────────────────────────────┐
│                                                               │
│  ┌─ data: Vec<i32> ─────────────────────────────────────┐   │
│  │  (owned, lines 5-40)                                  │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌─ { block } ───────────────────────────────────────────┐   │
│  │                                                        │   │
│  │  ┌─ r: &Vec<i32> ────────────────────────────────┐   │   │
│  │  │  (shared borrow of data, lines 8-22)           │   │   │
│  │  └────────────────────────────────────────────────┘   │   │
│  │                                                        │   │
│  │  ┌─ temp: i32 ───────────────────────────────────┐   │   │
│  │  │  (copy, lines 10-15)                           │   │   │
│  │  └────────────────────────────────────────────────┘   │   │
│  │                                                        │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌─ m: &mut Vec<i32> ───────────────────────────────────┐   │
│  │  (mutable borrow of data, lines 25-35)               │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                               │
│  Drop order: m → r → temp → data (reverse declaration)       │
└───────────────────────────────────────────────────────────────┘
```

**Steps:**
1. Build scope tree from function's block structure
2. Assign each variable to its containing scope
3. Render nested rectangles (outer = function, inner = blocks)
4. Show drop order annotation at the bottom
5. Color-code by ownership category

**Code (media/scopes.js):**
```javascript
function renderScopeView(data) {
    const { scopes, variables, dropOrder } = data;

    function renderScope(scope, container, depth) {
        const box = container.append('div')
            .attr('class', 'scope-box')
            .style('margin-left', `${depth * 20}px`)
            .style('border-color', scopeColor(scope.kind));

        box.append('div')
            .attr('class', 'scope-header')
            .text(`${scope.kind}: ${scope.name} (lines ${scope.startLine}-${scope.endLine})`);

        // Variables in this scope
        const vars = variables.filter(v => v.scopeId === scope.id);
        vars.forEach(v => {
            box.append('div')
                .attr('class', 'variable-entry')
                .style('border-left-color', categoryColor(v.category))
                .html(`<strong>${v.name}</strong>: ${v.type} <em>(${v.category})</em>`)
                .on('click', () => navigateToLine(v.line));
        });

        // Nested scopes
        const children = scopes.filter(s => s.parentId === scope.id);
        children.forEach(child => renderScope(child, box, depth + 1));
    }

    const root = scopes.find(s => s.kind === 'function');
    renderScope(root, d3.select('#scope-container'), 0);

    // Drop order footer
    d3.select('#scope-container').append('div')
        .attr('class', 'drop-order')
        .text(`Drop order: ${dropOrder.join(' → ')}`);
}
```

**Expectation:** The scope view answers "which variables are in the same scope?" and "what is the drop order?" at a glance. Nested blocks are visually nested.

**Tests for 7.2:**
- Function scope is the outermost box
- Block scopes nest inside function scope
- Variables appear in their correct scope
- Drop order is displayed (reverse declaration order)
- Clicking a variable navigates to its line
- Deeply nested scopes (3+ levels) render without overflow

---

## 7.3 Reference Count History

**Objective:** For Rc/Arc variables, display a line chart showing the reference count over the function's line range. Each clone increments the count, each drop decrements it. This visualizes shared ownership lifecycles.

**Visual design:**
```
Ref Count
    4 │            ┌───┐
    3 │        ┌───┘   │
    2 │    ┌───┘       └───┐
    1 │────┘               └───────
    0 │                            ────
      └────────────────────────────────── Line
       5   10   15   20   25   30   35

      rc created  rc2 cloned  rc3 cloned  rc2 dropped  rc3 dropped  rc dropped
      (count=1)   (count=2)   (count=3)   (count=2)    (count=1)    (count=0)
```

**Steps:**
1. Identify all Rc/Arc families (variables sharing the same allocation)
2. For each family, compute ref count at each relevant line
3. Render as a step chart (count changes at discrete points)
4. Annotate clone/drop events on the chart
5. Highlight if count never reaches 0 (potential leak)

**Code (media/refcount.js):**
```javascript
function renderRefCountChart(data) {
    const { families } = data;

    families.forEach(family => {
        const points = family.events.map(e => ({
            line: e.line,
            count: e.countAfter,
            event: e.kind, // 'created', 'cloned', 'dropped'
            variable: e.variable,
        }));

        const xScale = d3.scaleLinear()
            .domain([d3.min(points, p => p.line), d3.max(points, p => p.line)])
            .range([margin.left, width - margin.right]);

        const yScale = d3.scaleLinear()
            .domain([0, d3.max(points, p => p.count) + 1])
            .range([height - margin.bottom, margin.top]);

        // Step line
        const line = d3.line()
            .x(d => xScale(d.line))
            .y(d => yScale(d.count))
            .curve(d3.curveStepAfter);

        svg.append('path')
            .datum(points)
            .attr('d', line)
            .attr('fill', 'none')
            .attr('stroke', '#9b59b6')
            .attr('stroke-width', 2);

        // Event markers
        points.forEach(p => {
            svg.append('circle')
                .attr('cx', xScale(p.line))
                .attr('cy', yScale(p.count))
                .attr('r', 5)
                .attr('fill', p.event === 'dropped' ? '#e74c3c' : '#9b59b6')
                .on('mouseover', () => showTooltip(p));
        });

        // Leak warning
        const finalCount = points[points.length - 1]?.count || 0;
        if (finalCount > 0) {
            svg.append('text')
                .attr('x', width - margin.right)
                .attr('y', yScale(finalCount))
                .attr('fill', '#e74c3c')
                .attr('font-weight', 'bold')
                .text('⚠ LEAK: count never reaches 0');
        }
    });
}
```

**Expectation:** The chart makes Rc/Arc lifecycle immediately visible. Leaks (count never reaching 0) are highlighted with a warning.

**Tests for 7.3:**
- Single Rc (no clones): flat line at count=1, drops to 0
- Rc with 2 clones: steps up to 3, steps down to 0
- Leaked Rc: line stays above 0, warning displayed
- Multiple Rc families render as separate charts
- Hover on event marker shows variable name and action
- Chart scales correctly for functions with many clone/drop events

---

## 7.4 Move Chain View

**Objective:** Visualize the path a value takes through ownership transfers. Show the chain of variables that owned the value, with arrows indicating the direction and timing of each move.

**Visual design:**
```
┌─────────┐    move (L10)    ┌─────────┐    move (L20)    ┌─────────┐
│  input  │ ───────────────▶ │ parsed  │ ───────────────▶ │ result  │
│ String  │                  │ String  │                  │ String  │
│ L5-L10  │                  │ L10-L20 │                  │ L20-L35 │
└─────────┘                  └─────────┘                  └─────────┘
     ✗                            ✗                          (alive)
  invalidated                 invalidated
```

**Steps:**
1. For each move chain, trace from origin to final owner
2. Render as a horizontal sequence of boxes connected by arrows
3. Mark invalidated (moved-from) variables with ✗
4. Show the line number where each move occurs
5. Highlight the current owner (last in chain, still alive)

**Code (media/movechain.js):**
```javascript
function renderMoveChains(data) {
    const { chains } = data;

    chains.forEach((chain, chainIndex) => {
        const y = chainIndex * 80 + 40;

        chain.steps.forEach((step, i) => {
            const x = i * 200 + 50;

            // Node box
            const g = svg.append('g').attr('transform', `translate(${x}, ${y})`);
            g.append('rect')
                .attr('width', 140).attr('height', 50).attr('rx', 6)
                .attr('fill', step.isAlive ? '#2ecc71' : '#ecf0f1')
                .attr('stroke', step.isAlive ? '#27ae60' : '#bdc3c7')
                .attr('stroke-width', 2);

            g.append('text').attr('x', 70).attr('y', 20)
                .attr('text-anchor', 'middle').attr('font-weight', 'bold')
                .text(step.name);
            g.append('text').attr('x', 70).attr('y', 38)
                .attr('text-anchor', 'middle').attr('font-size', '10px')
                .text(`L${step.startLine}-L${step.endLine || '?'}`);

            // Invalidation marker
            if (!step.isAlive) {
                g.append('text').attr('x', 130).attr('y', 15)
                    .attr('fill', '#e74c3c').attr('font-size', '16px').text('✗');
            }

            // Arrow to next
            if (i < chain.steps.length - 1) {
                svg.append('line')
                    .attr('x1', x + 140).attr('y1', y + 25)
                    .attr('x2', x + 200).attr('y2', y + 25)
                    .attr('stroke', '#27ae60').attr('stroke-width', 2)
                    .attr('marker-end', 'url(#move-arrow)');
                svg.append('text')
                    .attr('x', x + 170).attr('y', y + 18)
                    .attr('text-anchor', 'middle').attr('font-size', '9px')
                    .attr('fill', '#27ae60')
                    .text(`move L${chain.moveLines[i]}`);
            }
        });
    });
}
```

**Expectation:** Move chains make ownership transfer paths immediately visible. The user can trace where a value came from and where it went.

**Tests for 7.4:**
- Single move (a→b): two boxes with one arrow
- Chain of 3 moves (a→b→c): three boxes with two arrows
- Moved-from variables show ✗ marker
- Final owner (alive) is highlighted green
- Move line numbers are displayed on arrows
- No moves: view shows "No ownership transfers in this function"

---

## 7.5 Conflict Highlight Mode

**Objective:** A toggle mode that dims the entire graph except for variables involved in borrow conflicts. Conflicting borrows are highlighted with red pulsing borders, and the conflict zone is marked with a red overlay.

**Steps:**
1. Add a "Show Conflicts" toggle button in the graph panel toolbar
2. When active, set all non-conflicting nodes/edges to 20% opacity
3. Highlight conflicting nodes with animated red border
4. Draw red zone overlay between conflicting borrow scopes
5. Show conflict explanation text

**Code (media/graph.js):**
```javascript
let conflictMode = false;

function toggleConflictMode() {
    conflictMode = !conflictMode;

    if (conflictMode && currentData.conflicts.length > 0) {
        const conflictVars = new Set();
        currentData.conflicts.forEach(c => {
            conflictVars.add(c.borrow_a.borrower);
            conflictVars.add(c.borrow_b.borrower);
            conflictVars.add(c.variable);
        });

        // Dim non-conflicting elements
        d3.selectAll('g.node')
            .style('opacity', d => conflictVars.has(d.name) ? 1 : 0.15);
        d3.selectAll('g.edge')
            .style('opacity', d => {
                const srcInConflict = conflictVars.has(findNode(d.source).name);
                const tgtInConflict = conflictVars.has(findNode(d.target).name);
                return (srcInConflict && tgtInConflict) ? 1 : 0.15;
            });

        // Pulse animation on conflict nodes
        d3.selectAll('g.node')
            .filter(d => conflictVars.has(d.name))
            .select('rect')
            .attr('stroke', '#e74c3c')
            .attr('stroke-width', 3)
            .attr('class', 'pulse-border');

    } else {
        // Restore normal view
        d3.selectAll('g.node').style('opacity', 1);
        d3.selectAll('g.edge').style('opacity', 1);
        d3.selectAll('g.node rect')
            .attr('stroke-width', 2)
            .attr('class', '');
    }
}
```

**Expectation:** Conflict mode provides instant focus on problematic ownership patterns. The user sees exactly which variables are involved and why.

**Tests for 7.5:**
- Toggle on: non-conflicting nodes fade to 15% opacity
- Toggle on: conflicting nodes get red pulsing border
- Toggle off: all nodes restore to full opacity
- No conflicts: toggle shows "No conflicts detected" message
- Multiple conflicts: all are highlighted simultaneously
- Conflict explanation text shows borrow types and overlap range

---

## 7.6 Comparison View

**Objective:** Show two ownership graphs side-by-side for comparing before/after states during refactoring. The user selects two git commits (or two points in undo history) and sees how the ownership structure changed.

**Steps:**
1. Add "Compare" command that opens a split WebView
2. Left panel shows the "before" graph, right panel shows "after"
3. Highlight differences: added nodes (green border), removed nodes (red strikethrough), changed edges
4. Show a summary: "+2 variables, -1 borrow, +1 move"

**Code (graph/comparison.ts):**
```typescript
export class ComparisonPanel {
    static show(before: FunctionOwnershipSummary, after: FunctionOwnershipSummary) {
        const panel = vscode.window.createWebviewPanel(
            'borrowscopeCompare',
            'BorrowScope: Compare',
            vscode.ViewColumn.Active,
            { enableScripts: true }
        );

        const diff = computeOwnershipDiff(before, after);
        panel.webview.html = getComparisonHtml(before, after, diff);
    }
}

interface OwnershipDiff {
    addedVariables: string[];
    removedVariables: string[];
    addedBorrows: BorrowScopeInfo[];
    removedBorrows: BorrowScopeInfo[];
    addedMoves: MoveInfo[];
    removedMoves: MoveInfo[];
    summary: string; // "+2 vars, -1 borrow, +1 move"
}
```

**Visual layout:**
```
┌─────────────────────────────┬─────────────────────────────┐
│  BEFORE (commit abc123)      │  AFTER (current)             │
│                              │                              │
│  ┌────┐     ┌────┐         │  ┌────┐     ┌────┐          │
│  │ a  │──&──│ r  │         │  │ a  │──&──│ r  │          │
│  └────┘     └────┘         │  └────┘     └────┘          │
│       │                     │       │     ┌────┐          │
│       └──&mut──┌────┐      │       └──&──│ r2 │ [+NEW]   │
│                │ m  │      │             └────┘          │
│                └────┘      │                              │
│                             │  Summary: +1 var, -1 &mut   │
└─────────────────────────────┴─────────────────────────────┘
```

**Expectation:** The comparison view makes refactoring impact visible. The user can see that changing `&mut` to `&` eliminated a conflict, or that extracting a function introduced a move.

**Tests for 7.6:**
- Side-by-side panels render both graphs
- Added nodes have green highlight
- Removed nodes have red strikethrough
- Summary shows correct counts of changes
- Identical graphs show "No ownership changes"
- Comparison works across different functions (before fn A, after fn B)

---

## 7.T Integration Test Suite

```typescript
suite('Advanced Visualization Tests', () => {
    test('Timeline renders for function with borrows', async () => {
        // Request timeline data, verify bars and scopes present
    });

    test('Scope view shows correct nesting', async () => {
        // Verify nested blocks appear as nested boxes
    });

    test('Ref count chart shows clone/drop events', async () => {
        // Verify step chart matches expected ref count history
    });

    test('Move chain traces ownership correctly', async () => {
        // Verify chain a→b→c renders with correct arrows
    });

    test('Conflict mode highlights only conflicting nodes', async () => {
        // Toggle conflict mode, verify opacity changes
    });

    test('Comparison view shows diff between two states', async () => {
        // Compare before/after, verify added/removed highlights
    });
});
```
