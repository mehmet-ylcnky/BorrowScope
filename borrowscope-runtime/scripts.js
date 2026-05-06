// Shared JavaScript functionality for all whitepaper sections
document.addEventListener('DOMContentLoaded', function() {
    // Smooth scrolling for table of contents links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });

    // Add syntax highlighting for inline code
    const codeElements = document.querySelectorAll('code');
    codeElements.forEach(code => {
        if (!code.classList.contains('rust-code')) {
            code.style.backgroundColor = 'var(--code-bg)';
            code.style.padding = '0.2em 0.4em';
            code.style.borderRadius = '3px';
            code.style.fontFamily = 'Consolas, Monaco, Courier New, monospace';
            code.style.fontSize = '0.9em';
        }
    });

    // Add copy functionality to code blocks
    document.querySelectorAll('.code-block').forEach(block => {
        const copyButton = document.createElement('button');
        copyButton.textContent = 'Copy';
        copyButton.className = 'copy-btn';
        
        const themeButton = document.createElement('button');
        themeButton.textContent = '🌙';
        themeButton.className = 'theme-btn';
        themeButton.title = 'Toggle dark/light theme';
        
        block.appendChild(copyButton);
        block.appendChild(themeButton);
        
        copyButton.addEventListener('click', () => {
            const text = block.querySelector('pre').textContent;
            navigator.clipboard.writeText(text).then(() => {
                copyButton.textContent = 'Copied!';
                setTimeout(() => {
                    copyButton.textContent = 'Copy';
                }, 2000);
            });
        });
        
        themeButton.addEventListener('click', () => {
            block.classList.toggle('dark-theme');
            themeButton.textContent = block.classList.contains('dark-theme') ? '☀️' : '🌙';
        });
    });

    // Initialize memory chart if container exists
    setTimeout(() => {
        if (document.getElementById('memory-chart')) {
            initializeMemoryChart();
        }
        if (document.getElementById('complex-pattern-chart')) {
            initializeComplexPatternChart();
        }
        if (document.getElementById('zero-cost-chart')) {
            initializeZeroCostChart();
        }
    }, 1000);
});

// Function to load section content
function loadSection(sectionFile, containerId) {
    fetch(sectionFile)
        .then(response => response.text())
        .then(html => {
            const parser = new DOMParser();
            const doc = parser.parseFromString(html, 'text/html');
            const content = doc.querySelector('.section-content');
            if (content) {
                document.getElementById(containerId).innerHTML = content.innerHTML;
                
                // Initialize memory chart if this section contains it
                setTimeout(() => {
                    if (document.getElementById('memory-chart')) {
                        initializeMemoryChart();
                    }
                    if (document.getElementById('complex-pattern-chart')) {
                        initializeComplexPatternChart();
                    }
                    if (document.getElementById('zero-cost-chart')) {
                        initializeZeroCostChart();
                    }
                }, 100);
            }
        })
        .catch(error => {
            console.error('Error loading section:', error);
            document.getElementById(containerId).innerHTML = '<p>Error loading section content.</p>';
        });
}

function initializeMemoryChart() {
    // Load D3.js if not already loaded
    if (typeof d3 === 'undefined') {
        const script = document.createElement('script');
        script.src = 'https://d3js.org/d3.v7.min.js';
        script.onload = createMemoryChart;
        document.head.appendChild(script);
    } else {
        createMemoryChart();
    }
}

function createMemoryChart() {
    const chartContainer = document.getElementById('memory-chart');
    if (!chartContainer) return;

    // Real data from benchmark results
    const memoryData = [
        { variables: 1000, events: 4000, memory_mb: 0.43 },
        { variables: 10000, events: 40000, memory_mb: 4.27 },
        { variables: 20000, events: 80000, memory_mb: 8.54 },
        { variables: 30000, events: 120000, memory_mb: 12.82 },
        { variables: 40000, events: 160000, memory_mb: 17.09 },
        { variables: 50000, events: 200000, memory_mb: 21.36 },
        { variables: 60000, events: 240000, memory_mb: 25.63 },
        { variables: 70000, events: 280000, memory_mb: 29.91 },
        { variables: 80000, events: 320000, memory_mb: 34.18 },
        { variables: 90000, events: 360000, memory_mb: 38.45 },
        { variables: 100000, events: 400000, memory_mb: 42.72 }
    ];

    // Chart dimensions - make it larger to fit the container better
    const chartMargin = { top: 30, right: 50, bottom: 80, left: 80 };
    const chartWidth = 800 - chartMargin.left - chartMargin.right;
    const chartHeight = 500 - chartMargin.top - chartMargin.bottom;

    // Clear any existing chart
    d3.select("#memory-chart").selectAll("*").remove();

    // Create SVG
    const chartSvg = d3.select("#memory-chart")
        .append("svg")
        .attr("width", chartWidth + chartMargin.left + chartMargin.right)
        .attr("height", chartHeight + chartMargin.top + chartMargin.bottom)
        .append("g")
        .attr("transform", `translate(${chartMargin.left},${chartMargin.top})`);

    // Scales
    const xScale = d3.scaleLinear()
        .domain([0, d3.max(memoryData, d => d.variables)])
        .range([0, chartWidth]);

    const yScale = d3.scaleLinear()
        .domain([0, d3.max(memoryData, d => d.memory_mb)])
        .range([chartHeight, 0]);

    // Grid lines
    chartSvg.append("g")
        .attr("transform", `translate(0,${chartHeight})`)
        .call(d3.axisBottom(xScale).tickSize(-chartHeight).tickFormat(""))
        .selectAll("line")
        .style("stroke", "#e0e0e0")
        .style("stroke-dasharray", "2,2");

    chartSvg.append("g")
        .call(d3.axisLeft(yScale).tickSize(-chartWidth).tickFormat(""))
        .selectAll("line")
        .style("stroke", "#e0e0e0")
        .style("stroke-dasharray", "2,2");

    // Line
    const line = d3.line()
        .x(d => xScale(d.variables))
        .y(d => yScale(d.memory_mb))
        .curve(d3.curveLinear);

    chartSvg.append("path")
        .datum(memoryData)
        .attr("fill", "none")
        .attr("stroke", "#3498db")
        .attr("stroke-width", 3)
        .attr("d", line);

    // Create tooltip
    const tooltip = d3.select("body").append("div")
        .style("position", "absolute")
        .style("background", "rgba(0, 0, 0, 0.8)")
        .style("color", "white")
        .style("padding", "8px 12px")
        .style("border-radius", "4px")
        .style("font-size", "12px")
        .style("pointer-events", "none")
        .style("opacity", 0)
        .style("z-index", 1000);

    // Dots with tooltip
    chartSvg.selectAll(".dot")
        .data(memoryData)
        .enter().append("circle")
        .attr("cx", d => xScale(d.variables))
        .attr("cy", d => yScale(d.memory_mb))
        .attr("r", 4)
        .attr("fill", "#e74c3c")
        .attr("stroke", "#c0392b")
        .attr("stroke-width", 2)
        .on("mouseover", function(event, d) {
            d3.select(this).attr("r", 6).attr("fill", "#f39c12");
            tooltip.transition().duration(200).style("opacity", .9);
            tooltip.html(`
                <strong>${d.variables.toLocaleString()} Variables</strong><br/>
                Events: ${d.events.toLocaleString()}<br/>
                Memory: ${d.memory_mb.toFixed(2)} MB<br/>
                Efficiency: ${(d.memory_mb * 1024 / d.variables).toFixed(0)} KB/var
            `)
                .style("left", (event.pageX + 10) + "px")
                .style("top", (event.pageY - 28) + "px");
        })
        .on("mouseout", function(d) {
            d3.select(this).attr("r", 4).attr("fill", "#e74c3c");
            tooltip.transition().duration(500).style("opacity", 0);
        });

    // Axes
    chartSvg.append("g")
        .attr("transform", `translate(0,${chartHeight})`)
        .call(d3.axisBottom(xScale).tickFormat(d => d >= 1000 ? (d/1000) + "K" : d))
        .style("font-size", "12px");

    chartSvg.append("g")
        .call(d3.axisLeft(yScale))
        .style("font-size", "12px");

    // Labels
    chartSvg.append("text")
        .attr("transform", `translate(${chartWidth/2}, ${chartHeight + 45})`)
        .style("text-anchor", "middle")
        .style("font-size", "14px")
        .style("font-weight", "bold")
        .text("Number of Variables");

    chartSvg.append("text")
        .attr("transform", "rotate(-90)")
        .attr("y", 0 - chartMargin.left + 20)
        .attr("x", 0 - (chartHeight / 2))
        .style("text-anchor", "middle")
        .style("font-size", "14px")
        .style("font-weight", "bold")
        .text("Memory Usage (MB)");

    // Trend line equation
    chartSvg.append("text")
        .attr("x", chartWidth - 180)
        .attr("y", 30)
        .style("font-size", "12px")
        .style("fill", "#3498db")
        .text("y = 0.427x + 0.003");

    chartSvg.append("text")
        .attr("x", chartWidth - 180)
        .attr("y", 45)
        .style("font-size", "10px")
        .style("fill", "#7f8c8d")
        .text("Perfect Linear Relationship");
}

function initializeComplexPatternChart() {
    // Load D3.js if not already loaded
    if (typeof d3 === 'undefined') {
        const script = document.createElement('script');
        script.src = 'https://d3js.org/d3.v7.min.js';
        script.onload = createComplexPatternChart;
        document.head.appendChild(script);
    } else {
        createComplexPatternChart();
    }
}

function createComplexPatternChart() {
    const chartContainer = document.getElementById('complex-pattern-chart');
    if (!chartContainer) return;

    // Complex pattern benchmark data
    const patternData = [
        { operation: 'Rc Creation', count: 1, time_ns: 90, color: '#3498db' },
        { operation: 'Rc Cloning', count: 2, time_ns: 85, color: '#2980b9' },
        { operation: 'RefCell Creation', count: 1, time_ns: 95, color: '#e74c3c' },
        { operation: 'RefCell Borrow', count: 1, time_ns: 95, color: '#c0392b' },
        { operation: 'RefCell BorrowMut', count: 1, time_ns: 100, color: '#a93226' },
        { operation: 'Drop Operations', count: 6, time_ns: 80, color: '#f39c12' }
    ];

    // Chart dimensions
    const margin = { top: 30, right: 50, bottom: 80, left: 80 };
    const width = 700 - margin.left - margin.right;
    const height = 350 - margin.top - margin.bottom;

    // Clear any existing chart
    d3.select("#complex-pattern-chart").selectAll("*").remove();

    // Create SVG
    const svg = d3.select("#complex-pattern-chart")
        .append("svg")
        .attr("width", width + margin.left + margin.right)
        .attr("height", height + margin.top + margin.bottom)
        .append("g")
        .attr("transform", `translate(${margin.left},${margin.top})`);

    // Scales
    const xScale = d3.scaleBand()
        .domain(patternData.map(d => d.operation))
        .range([0, width])
        .padding(0.2);

    const yScale = d3.scaleLinear()
        .domain([0, d3.max(patternData, d => d.time_ns) * 1.1])
        .range([height, 0]);

    // Create tooltip
    const tooltip = d3.select("body").append("div")
        .style("position", "absolute")
        .style("background", "rgba(0, 0, 0, 0.8)")
        .style("color", "white")
        .style("padding", "8px 12px")
        .style("border-radius", "4px")
        .style("font-size", "12px")
        .style("pointer-events", "none")
        .style("opacity", 0)
        .style("z-index", 1000);

    // Bars
    svg.selectAll(".bar")
        .data(patternData)
        .enter().append("rect")
        .attr("class", "bar")
        .attr("x", d => xScale(d.operation))
        .attr("width", xScale.bandwidth())
        .attr("y", d => yScale(d.time_ns))
        .attr("height", d => height - yScale(d.time_ns))
        .attr("fill", d => d.color)
        .attr("stroke", "#2c3e50")
        .attr("stroke-width", 1)
        .on("mouseover", function(event, d) {
            d3.select(this).attr("opacity", 0.8);
            tooltip.transition().duration(200).style("opacity", .9);
            tooltip.html(`
                <strong>${d.operation}</strong><br/>
                Count per cycle: ${d.count}<br/>
                Time per operation: ${d.time_ns} ns<br/>
                Total per cycle: ${d.count * d.time_ns} ns
            `)
                .style("left", (event.pageX + 10) + "px")
                .style("top", (event.pageY - 28) + "px");
        })
        .on("mouseout", function(d) {
            d3.select(this).attr("opacity", 1);
            tooltip.transition().duration(500).style("opacity", 0);
        });

    // Add value labels on bars
    svg.selectAll(".label")
        .data(patternData)
        .enter().append("text")
        .attr("class", "label")
        .attr("x", d => xScale(d.operation) + xScale.bandwidth() / 2)
        .attr("y", d => yScale(d.time_ns) - 5)
        .attr("text-anchor", "middle")
        .style("font-size", "11px")
        .style("font-weight", "bold")
        .style("fill", "#2c3e50")
        .text(d => `${d.time_ns}ns`);

    // X axis
    svg.append("g")
        .attr("transform", `translate(0,${height})`)
        .call(d3.axisBottom(xScale))
        .selectAll("text")
        .style("font-size", "11px")
        .attr("transform", "rotate(-45)")
        .style("text-anchor", "end");

    // Y axis
    svg.append("g")
        .call(d3.axisLeft(yScale))
        .style("font-size", "12px");

    // Y axis label
    svg.append("text")
        .attr("transform", "rotate(-90)")
        .attr("y", 0 - margin.left + 20)
        .attr("x", 0 - (height / 2))
        .style("text-anchor", "middle")
        .style("font-size", "14px")
        .style("font-weight", "bold")
        .text("Time per Operation (nanoseconds)");

    // Title
    svg.append("text")
        .attr("x", width / 2)
        .attr("y", 0 - (margin.top / 2))
        .attr("text-anchor", "middle")
        .style("font-size", "16px")
        .style("font-weight", "bold")
        .style("fill", "#2c3e50")
        .text("Complex Pattern Performance Breakdown");
}

function initializeZeroCostChart() {
    // Load D3.js if not already loaded
    if (typeof d3 === 'undefined') {
        const script = document.createElement('script');
        script.src = 'https://d3js.org/d3.v7.min.js';
        script.onload = createZeroCostChart;
        document.head.appendChild(script);
    } else {
        createZeroCostChart();
    }
}

function createZeroCostChart() {
    const chartContainer = document.getElementById('zero-cost-chart');
    if (!chartContainer) return;

    // Zero-cost verification data
    const zeroCostData = [
        { 
            test: 'Basic Operations', 
            enabled: 119.9, 
            disabled: 18.4, 
            unit: 'ms',
            improvement: '84.6% faster'
        },
        { 
            test: 'Complex Patterns', 
            enabled: 1.09, 
            disabled: 0.41, 
            unit: 'ms',
            improvement: '62.4% faster'
        },
        { 
            test: 'Memory Events', 
            enabled: 600000, 
            disabled: 0, 
            unit: 'events',
            improvement: '100% eliminated'
        },
        { 
            test: 'Memory Overhead', 
            enabled: 65.6, 
            disabled: 0, 
            unit: 'MB',
            improvement: '100% eliminated'
        }
    ];

    // Chart dimensions - make wider to accommodate text
    const margin = { top: 40, right: 80, bottom: 80, left: 80 };
    const width = 900 - margin.left - margin.right;
    const height = 450 - margin.top - margin.bottom;

    // Clear any existing chart
    d3.select("#zero-cost-chart").selectAll("*").remove();

    // Create SVG
    const svg = d3.select("#zero-cost-chart")
        .append("svg")
        .attr("width", width + margin.left + margin.right)
        .attr("height", height + margin.top + margin.bottom)
        .append("g")
        .attr("transform", `translate(${margin.left},${margin.top})`);

    // Scales
    const x0Scale = d3.scaleBand()
        .domain(zeroCostData.map(d => d.test))
        .range([0, width])
        .padding(0.3);

    const x1Scale = d3.scaleBand()
        .domain(['enabled', 'disabled'])
        .range([0, x0Scale.bandwidth()])
        .padding(0.1);

    // Use different scales for different metrics
    const yScales = {
        'ms': d3.scaleLinear().domain([0, d3.max(zeroCostData.slice(0,2), d => Math.max(d.enabled, d.disabled)) * 1.1]).range([height, 0]),
        'events': d3.scaleLinear().domain([0, 600000 * 1.1]).range([height, 0]),
        'MB': d3.scaleLinear().domain([0, 70]).range([height, 0])
    };

    // Create tooltip
    const tooltip = d3.select("body").append("div")
        .style("position", "absolute")
        .style("background", "rgba(0, 0, 0, 0.8)")
        .style("color", "white")
        .style("padding", "8px 12px")
        .style("border-radius", "4px")
        .style("font-size", "12px")
        .style("pointer-events", "none")
        .style("opacity", 0)
        .style("z-index", 1000);

    // Create groups for each test
    const testGroups = svg.selectAll(".test-group")
        .data(zeroCostData)
        .enter().append("g")
        .attr("class", "test-group")
        .attr("transform", d => `translate(${x0Scale(d.test)},0)`);

    // Add enabled bars
    testGroups.append("rect")
        .attr("class", "enabled-bar")
        .attr("x", x1Scale('enabled'))
        .attr("width", x1Scale.bandwidth())
        .attr("y", d => {
            const scale = d.unit === 'events' ? yScales.events : (d.unit === 'MB' ? yScales.MB : yScales.ms);
            return scale(d.enabled);
        })
        .attr("height", d => {
            const scale = d.unit === 'events' ? yScales.events : (d.unit === 'MB' ? yScales.MB : yScales.ms);
            return height - scale(d.enabled);
        })
        .attr("fill", "#e74c3c")
        .attr("stroke", "#c0392b")
        .attr("stroke-width", 1)
        .on("mouseover", function(event, d) {
            tooltip.transition().duration(200).style("opacity", .9);
            tooltip.html(`
                <strong>Tracking Enabled</strong><br/>
                ${d.test}<br/>
                Value: ${d.enabled.toLocaleString()} ${d.unit}<br/>
                Performance: Baseline
            `)
                .style("left", (event.pageX + 10) + "px")
                .style("top", (event.pageY - 28) + "px");
        })
        .on("mouseout", function() {
            tooltip.transition().duration(500).style("opacity", 0);
        });

    // Add disabled bars
    testGroups.append("rect")
        .attr("class", "disabled-bar")
        .attr("x", x1Scale('disabled'))
        .attr("width", x1Scale.bandwidth())
        .attr("y", d => {
            const scale = d.unit === 'events' ? yScales.events : (d.unit === 'MB' ? yScales.MB : yScales.ms);
            return scale(d.disabled);
        })
        .attr("height", d => {
            const scale = d.unit === 'events' ? yScales.events : (d.unit === 'MB' ? yScales.MB : yScales.ms);
            return height - scale(d.disabled);
        })
        .attr("fill", "#27ae60")
        .attr("stroke", "#229954")
        .attr("stroke-width", 1)
        .on("mouseover", function(event, d) {
            tooltip.transition().duration(200).style("opacity", .9);
            tooltip.html(`
                <strong>Tracking Disabled</strong><br/>
                ${d.test}<br/>
                Value: ${d.disabled.toLocaleString()} ${d.unit}<br/>
                Improvement: ${d.improvement}
            `)
                .style("left", (event.pageX + 10) + "px")
                .style("top", (event.pageY - 28) + "px");
        })
        .on("mouseout", function() {
            tooltip.transition().duration(500).style("opacity", 0);
        });

    // Add improvement labels - move them down
    testGroups.append("text")
        .attr("x", x0Scale.bandwidth() / 2)
        .attr("y", 15)
        .attr("text-anchor", "middle")
        .style("font-size", "11px")
        .style("font-weight", "bold")
        .style("fill", "#27ae60")
        .text(d => d.improvement);

    // X axis
    svg.append("g")
        .attr("transform", `translate(0,${height})`)
        .call(d3.axisBottom(x0Scale))
        .selectAll("text")
        .style("font-size", "11px")
        .style("text-anchor", "middle");

    // Y axis label
    svg.append("text")
        .attr("transform", "rotate(-90)")
        .attr("y", 0 - margin.left + 20)
        .attr("x", 0 - (height / 2))
        .style("text-anchor", "middle")
        .style("font-size", "14px")
        .style("font-weight", "bold")
        .text("Performance Metrics (Various Units)");

    // Legend - move to the right outside chart area
    const legend = svg.append("g")
        .attr("transform", `translate(${width + 20}, 50)`);

    legend.append("rect")
        .attr("x", 0)
        .attr("y", 0)
        .attr("width", 15)
        .attr("height", 15)
        .attr("fill", "#e74c3c");

    legend.append("text")
        .attr("x", 20)
        .attr("y", 12)
        .style("font-size", "12px")
        .text("Tracking Enabled");

    legend.append("rect")
        .attr("x", 0)
        .attr("y", 25)
        .attr("width", 15)
        .attr("height", 15)
        .attr("fill", "#27ae60");

    legend.append("text")
        .attr("x", 20)
        .attr("y", 37)
        .style("font-size", "12px")
        .text("Tracking Disabled");

    // Title
    svg.append("text")
        .attr("x", width / 2)
        .attr("y", 0 - (margin.top / 2))
        .attr("text-anchor", "middle")
        .style("font-size", "16px")
        .style("font-weight", "bold")
        .style("fill", "#2c3e50")
        .text("Zero-Cost Abstraction Verification");
}


// Event Taxonomy Explorer - Global functions for inline handlers
window.eventData = {
    cat1: { events: { "New": { desc: "Variable creation (ownership begins)", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String" }, "Borrow": { desc: "Borrow creation (shared or mutable reference)", fields: "timestamp: u64, borrower_name: String, borrower_id: String, owner_id: String, mutable: bool" }, "Move": { desc: "Ownership transfer (move semantics)", fields: "timestamp: u64, from_id: String, to_name: String, to_id: String" }, "Drop": { desc: "Variable destruction (ownership ends)", fields: "timestamp: u64, var_id: String" } } },
    cat2: { events: { "RcNew": { desc: "Rc<T> allocation", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String, strong_count: usize, weak_count: usize" }, "RcClone": { desc: "Rc<T> clone (shared ownership)", fields: "timestamp: u64, var_name: String, var_id: String, source_id: String, strong_count: usize, weak_count: usize" }, "ArcNew": { desc: "Arc<T> allocation (thread-safe)", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String, strong_count: usize, weak_count: usize" }, "ArcClone": { desc: "Arc<T> clone (thread-safe shared ownership)", fields: "timestamp: u64, var_name: String, var_id: String, source_id: String, strong_count: usize, weak_count: usize" } } },
    cat3: { events: { "RefCellNew": { desc: "RefCell<T> allocation", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String" }, "RefCellBorrow": { desc: "RefCell borrow (runtime borrow checking)", fields: "timestamp: u64, borrow_id: String, refcell_id: String, is_mutable: bool, location: String" }, "RefCellDrop": { desc: "RefCell borrow released", fields: "timestamp: u64, refcell_id: String" }, "CellNew": { desc: "Cell<T> allocation", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String" }, "CellGet": { desc: "Cell value read (Copy)", fields: "timestamp: u64, var_id: String" }, "CellSet": { desc: "Cell value write", fields: "timestamp: u64, var_id: String" } } },
    cat4: { events: { "RawPtrCreated": { desc: "Raw pointer creation", fields: "timestamp: u64, var_name: String, var_id: String, ptr_type: String, address: usize, location: String" }, "RawPtrDeref": { desc: "Raw pointer dereference", fields: "timestamp: u64, ptr_id: String, location: String" }, "UnsafeBlockEnter": { desc: "Entering unsafe block", fields: "timestamp: u64, block_id: String, location: String" }, "UnsafeBlockExit": { desc: "Exiting unsafe block", fields: "timestamp: u64, block_id: String, location: String" }, "UnsafeFnCall": { desc: "Unsafe function call", fields: "timestamp: u64, fn_name: String, location: String" }, "FfiCall": { desc: "FFI function call", fields: "timestamp: u64, fn_name: String, location: String" }, "Transmute": { desc: "Type transmutation", fields: "timestamp: u64, from_type: String, to_type: String, location: String" }, "UnionFieldAccess": { desc: "Union field access", fields: "timestamp: u64, union_id: String, field_name: String" } } },
    cat5: { events: { "StaticInit": { desc: "Static variable initialization", fields: "timestamp: u64, name: String, type_name: String" }, "StaticAccess": { desc: "Static variable access", fields: "timestamp: u64, name: String, is_mutable: bool" }, "ConstEval": { desc: "Const evaluation", fields: "timestamp: u64, name: String, type_name: String" } } },
    cat6: { events: { "AsyncBlockEnter": { desc: "Entering async block", fields: "timestamp: u64, block_id: String" }, "AsyncBlockExit": { desc: "Exiting async block", fields: "timestamp: u64, block_id: String" }, "AwaitStart": { desc: "Await point reached", fields: "timestamp: u64, future_id: String" }, "AwaitEnd": { desc: "Await completed", fields: "timestamp: u64, future_id: String" } } },
    cat7: { events: { "BoxNew": { desc: "Box<T> heap allocation", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String" }, "BoxIntoRaw": { desc: "Box converted to raw pointer", fields: "timestamp: u64, var_id: String, ptr_addr: usize" }, "BoxFromRaw": { desc: "Box reconstructed from raw pointer", fields: "timestamp: u64, var_name: String, var_id: String, ptr_addr: usize" } } },
    cat8: { events: { "WeakNew": { desc: "Weak<T> creation from Rc/Arc", fields: "timestamp: u64, var_name: String, var_id: String, source_id: String" }, "WeakClone": { desc: "Weak<T> clone", fields: "timestamp: u64, var_name: String, var_id: String, source_id: String" }, "WeakUpgrade": { desc: "Weak<T> upgrade attempt", fields: "timestamp: u64, var_name: String, var_id: String, source_id: String, success: bool" } } },
    cat9: { events: { "PinNew": { desc: "Pin<T> creation (immovable guarantee)", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String" }, "PinIntoInner": { desc: "Pin unwrapped to inner value", fields: "timestamp: u64, var_id: String" } } },
    cat10: { events: { "CowBorrowed": { desc: "Cow<T> created with borrowed data", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String" }, "CowOwned": { desc: "Cow<T> created with owned data", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String" }, "CowToMut": { desc: "Cow<T> converted to mutable (clone-on-write)", fields: "timestamp: u64, var_id: String, was_borrowed: bool" } } },
    cat11: { events: { "ThreadSpawn": { desc: "Thread spawned", fields: "timestamp: u64, thread_id: String, thread_name: Option<String>" }, "ThreadJoin": { desc: "Thread joined", fields: "timestamp: u64, thread_id: String" }, "ChannelSenderNew": { desc: "Channel sender created", fields: "timestamp: u64, var_name: String, var_id: String, channel_id: String" }, "ChannelReceiverNew": { desc: "Channel receiver created", fields: "timestamp: u64, var_name: String, var_id: String, channel_id: String" }, "ChannelSend": { desc: "Message sent through channel", fields: "timestamp: u64, sender_id: String, channel_id: String" }, "ChannelRecv": { desc: "Message received from channel", fields: "timestamp: u64, receiver_id: String, channel_id: String, success: bool" } } },
    cat12: { events: { "LockGuardAcquire": { desc: "Lock guard acquired", fields: "timestamp: u64, guard_name: String, guard_id: String, lock_id: String, lock_type: String" }, "LockGuardDrop": { desc: "Lock guard released", fields: "timestamp: u64, guard_id: String" } } },
    cat13: { events: { "OnceCellNew": { desc: "OnceCell/OnceLock creation", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String, is_sync: bool" }, "OnceCellSet": { desc: "OnceCell/OnceLock initialization", fields: "timestamp: u64, var_id: String, success: bool" }, "OnceCellGet": { desc: "OnceCell/OnceLock value access", fields: "timestamp: u64, var_id: String, was_initialized: bool" }, "OnceCellGetOrInit": { desc: "OnceCell/OnceLock get or initialize", fields: "timestamp: u64, var_id: String, was_initialized: bool" } } },
    cat14: { events: { "MaybeUninitNew": { desc: "MaybeUninit<T> creation", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String, is_initialized: bool" }, "MaybeUninitWrite": { desc: "Write to MaybeUninit", fields: "timestamp: u64, var_id: String" }, "MaybeUninitAssumeInit": { desc: "Assume initialized (ownership transfer)", fields: "timestamp: u64, var_id: String, new_var_name: String, new_var_id: String" }, "MaybeUninitAssumeInitRead": { desc: "Read assuming initialized", fields: "timestamp: u64, var_id: String" }, "MaybeUninitAssumeInitDrop": { desc: "Drop assuming initialized", fields: "timestamp: u64, var_id: String" } } },
    cat15: { events: { "LoopEnter": { desc: "Loop entry", fields: "timestamp: u64, loop_id: String, loop_type: String" }, "LoopIteration": { desc: "Loop iteration", fields: "timestamp: u64, loop_id: String, iteration: u64" }, "LoopExit": { desc: "Loop exit", fields: "timestamp: u64, loop_id: String" }, "MatchEnter": { desc: "Match expression entry", fields: "timestamp: u64, match_id: String" }, "MatchArm": { desc: "Match arm selected", fields: "timestamp: u64, match_id: String, arm_index: usize" }, "MatchExit": { desc: "Match expression exit", fields: "timestamp: u64, match_id: String" }, "Branch": { desc: "Conditional branch", fields: "timestamp: u64, branch_id: String, condition: bool" }, "Return": { desc: "Function return", fields: "timestamp: u64, fn_id: String" }, "Try": { desc: "Try operator (?)", fields: "timestamp: u64, expr_id: String, success: bool" }, "IndexAccess": { desc: "Index access operation", fields: "timestamp: u64, container_id: String, index: String" }, "FieldAccess": { desc: "Field access operation", fields: "timestamp: u64, struct_id: String, field_name: String" }, "Call": { desc: "Function/method call", fields: "timestamp: u64, fn_name: String, call_id: String" }, "Lock": { desc: "Lock acquisition", fields: "timestamp: u64, lock_id: String, lock_type: String" }, "Unwrap": { desc: "Unwrap operation", fields: "timestamp: u64, expr_id: String, success: bool" }, "Clone": { desc: "Clone operation", fields: "timestamp: u64, source_id: String, clone_id: String" }, "Deref": { desc: "Dereference operation", fields: "timestamp: u64, ref_id: String" }, "Break": { desc: "Break statement", fields: "timestamp: u64, loop_id: String" }, "Continue": { desc: "Continue statement", fields: "timestamp: u64, loop_id: String" }, "ClosureCreate": { desc: "Closure creation", fields: "timestamp: u64, closure_id: String, capture_count: usize" }, "StructCreate": { desc: "Struct instantiation", fields: "timestamp: u64, var_name: String, var_id: String, type_name: String" }, "TupleCreate": { desc: "Tuple creation", fields: "timestamp: u64, var_name: String, var_id: String, element_count: usize" }, "LetElse": { desc: "Let-else pattern", fields: "timestamp: u64, var_id: String, matched: bool" }, "Range": { desc: "Range expression", fields: "timestamp: u64, range_id: String, range_type: String" }, "BinaryOp": { desc: "Binary operation", fields: "timestamp: u64, op: String, lhs_id: String, rhs_id: String" }, "ArrayCreate": { desc: "Array creation", fields: "timestamp: u64, var_name: String, var_id: String, length: usize" }, "TypeCast": { desc: "Type cast (as)", fields: "timestamp: u64, expr_id: String, from_type: String, to_type: String" }, "RegionEnter": { desc: "Scope/region entry", fields: "timestamp: u64, region_id: String" }, "RegionExit": { desc: "Scope/region exit", fields: "timestamp: u64, region_id: String" }, "FnEnter": { desc: "Function entry", fields: "timestamp: u64, fn_name: String, fn_id: String" }, "FnExit": { desc: "Function exit", fields: "timestamp: u64, fn_id: String" }, "ClosureCapture": { desc: "Closure variable capture", fields: "timestamp: u64, closure_id: String, var_id: String, capture_type: String" } } }
};

window.updateEventDropdown = function(cat) {
    var eventSelect = document.getElementById('event-select');
    var eventDetails = document.getElementById('event-details');
    if (!eventSelect) return;
    
    eventSelect.innerHTML = '<option value="">-- Select Event --</option>';
    if (eventDetails) eventDetails.style.display = 'none';
    
    if (cat && window.eventData[cat]) {
        eventSelect.removeAttribute('disabled');
        for (var evt in window.eventData[cat].events) {
            var opt = document.createElement('option');
            opt.value = evt;
            opt.textContent = evt;
            eventSelect.appendChild(opt);
        }
    } else {
        eventSelect.setAttribute('disabled', 'disabled');
    }
};

window.showEventDetails = function() {
    var catSelect = document.getElementById('category-select');
    var eventSelect = document.getElementById('event-select');
    var eventDetails = document.getElementById('event-details');
    var eventCode = document.getElementById('event-code');
    if (!catSelect || !eventSelect || !eventDetails || !eventCode) return;
    
    var cat = catSelect.value;
    var evt = eventSelect.value;
    
    if (cat && evt && window.eventData[cat] && window.eventData[cat].events[evt]) {
        var data = window.eventData[cat].events[evt];
        var fields = data.fields.split(', ').map(function(f) {
            var parts = f.split(': ');
            return '<span class="rust-keyword">' + parts[0] + '</span>: <span class="rust-type">' + parts[1] + '</span>';
        }).join(',\n    ');
        eventCode.innerHTML = '<span class="rust-comment">// ' + data.desc + '</span>\n<span class="rust-type">' + evt + '</span> {\n    ' + fields + '\n}';
        eventDetails.style.display = 'block';
    } else {
        eventDetails.style.display = 'none';
    }
};


// Tracking API Explorer - Global functions for inline handlers
window.apiData = {
    api1: { fns: { "track_new": "pub fn track_new<T>(name: &str, value: T) -> T", "track_new_with_id": "pub fn track_new_with_id<T>(name: &str, id: &str, value: T) -> T", "track_borrow": "pub fn track_borrow<'a, T: ?Sized>(name: &str, reference: &'a T) -> &'a T", "track_borrow_with_id": "pub fn track_borrow_with_id<'a, T: ?Sized>(name: &str, id: &str, owner_id: &str, reference: &'a T) -> &'a T", "track_borrow_mut": "pub fn track_borrow_mut<'a, T: ?Sized>(name: &str, reference: &'a mut T) -> &'a mut T", "track_borrow_mut_with_id": "pub fn track_borrow_mut_with_id<'a, T: ?Sized>(name: &str, id: &str, owner_id: &str, reference: &'a mut T) -> &'a mut T", "track_move": "pub fn track_move<T>(name: &str, value: T) -> T", "track_move_with_id": "pub fn track_move_with_id<T>(name: &str, id: &str, from_id: &str, value: T) -> T", "track_drop": "pub fn track_drop(name: &str)", "track_drop_with_id": "pub fn track_drop_with_id(id: &str)", "track_drop_batch": "pub fn track_drop_batch(names: &[&str])" } },
    api2: { fns: { "track_rc_new": "pub fn track_rc_new<T: ?Sized>(name: &str, rc: Rc<T>) -> Rc<T>", "track_rc_new_with_id": "pub fn track_rc_new_with_id<T: ?Sized>(name: &str, id: &str, rc: Rc<T>) -> Rc<T>", "track_rc_clone": "pub fn track_rc_clone<T: ?Sized>(name: &str, source: &str, rc: Rc<T>) -> Rc<T>", "track_rc_clone_with_id": "pub fn track_rc_clone_with_id<T: ?Sized>(name: &str, id: &str, source_id: &str, rc: Rc<T>) -> Rc<T>", "track_arc_new": "pub fn track_arc_new<T: ?Sized>(name: &str, arc: Arc<T>) -> Arc<T>", "track_arc_new_with_id": "pub fn track_arc_new_with_id<T: ?Sized>(name: &str, id: &str, arc: Arc<T>) -> Arc<T>", "track_arc_clone": "pub fn track_arc_clone<T: ?Sized>(name: &str, source: &str, arc: Arc<T>) -> Arc<T>", "track_arc_clone_with_id": "pub fn track_arc_clone_with_id<T: ?Sized>(name: &str, id: &str, source_id: &str, arc: Arc<T>) -> Arc<T>" } },
    api3: { fns: { "track_refcell_new": "pub fn track_refcell_new<T>(name: &str, cell: RefCell<T>) -> RefCell<T>", "track_refcell_borrow": "pub fn track_refcell_borrow<'a, T>(name: &str, guard: Ref<'a, T>) -> Ref<'a, T>", "track_refcell_borrow_mut": "pub fn track_refcell_borrow_mut<'a, T>(name: &str, guard: RefMut<'a, T>) -> RefMut<'a, T>", "track_refcell_drop": "pub fn track_refcell_drop(name: &str)", "track_cell_new": "pub fn track_cell_new<T>(name: &str, cell: Cell<T>) -> Cell<T>", "track_cell_get": "pub fn track_cell_get<T: Copy>(name: &str, value: T) -> T", "track_cell_set": "pub fn track_cell_set(name: &str)" } },
    api4: { fns: { "track_raw_ptr": "pub fn track_raw_ptr<T: ?Sized>(name: &str, ptr: *const T) -> *const T", "track_raw_ptr_mut": "pub fn track_raw_ptr_mut<T: ?Sized>(name: &str, ptr: *mut T) -> *mut T", "track_raw_ptr_deref": "pub fn track_raw_ptr_deref(name: &str)", "track_unsafe_block_enter": "pub fn track_unsafe_block_enter(block_id: &str)", "track_unsafe_block_exit": "pub fn track_unsafe_block_exit(block_id: &str)", "track_unsafe_fn_call": "pub fn track_unsafe_fn_call(fn_name: &str)", "track_ffi_call": "pub fn track_ffi_call(fn_name: &str)", "track_transmute": "pub fn track_transmute(from_type: &str, to_type: &str)", "track_union_field_access": "pub fn track_union_field_access(union_name: &str, field: &str)" } },
    api5: { fns: { "track_static_init": "pub fn track_static_init<T>(name: &str, value: T) -> T", "track_static_access": "pub fn track_static_access(name: &str, is_mutable: bool)", "track_const_eval": "pub fn track_const_eval<T>(name: &str, value: T) -> T" } },
    api6: { fns: { "track_async_block_enter": "pub fn track_async_block_enter(block_id: &str)", "track_async_block_exit": "pub fn track_async_block_exit(block_id: &str)", "track_await_start": "pub fn track_await_start(future_id: &str)", "track_await_end": "pub fn track_await_end(future_id: &str)" } },
    api7: { fns: { "track_box_new": "pub fn track_box_new<T: ?Sized>(name: &str, boxed: Box<T>) -> Box<T>", "track_box_into_raw": "pub fn track_box_into_raw<T>(name: &str, ptr: *mut T) -> *mut T", "track_box_from_raw": "pub unsafe fn track_box_from_raw<T>(name: &str, boxed: Box<T>) -> Box<T>" } },
    api8: { fns: { "track_weak_new": "pub fn track_weak_new<T: ?Sized>(name: &str, source: &str, weak: Weak<T>) -> Weak<T>", "track_weak_new_sync": "pub fn track_weak_new_sync<T: ?Sized>(name: &str, source: &str, weak: sync::Weak<T>) -> sync::Weak<T>", "track_weak_clone": "pub fn track_weak_clone<T: ?Sized>(name: &str, source: &str, weak: Weak<T>) -> Weak<T>", "track_weak_clone_sync": "pub fn track_weak_clone_sync<T: ?Sized>(name: &str, source: &str, weak: sync::Weak<T>) -> sync::Weak<T>", "track_weak_upgrade": "pub fn track_weak_upgrade<T>(name: &str, source: &str, rc: Option<Rc<T>>) -> Option<Rc<T>>", "track_weak_upgrade_sync": "pub fn track_weak_upgrade_sync<T>(name: &str, source: &str, arc: Option<Arc<T>>) -> Option<Arc<T>>" } },
    api9: { fns: { "track_pin_new": "pub fn track_pin_new<P>(name: &str, pin: Pin<P>) -> Pin<P>", "track_pin_into_inner": "pub unsafe fn track_pin_into_inner<T>(name: &str, value: T) -> T" } },
    api10: { fns: { "track_cow_borrowed": "pub fn track_cow_borrowed<'a, B: ?Sized + ToOwned>(name: &str, cow: Cow<'a, B>) -> Cow<'a, B>", "track_cow_owned": "pub fn track_cow_owned<'a, B: ?Sized + ToOwned>(name: &str, cow: Cow<'a, B>) -> Cow<'a, B>", "track_cow_to_mut": "pub fn track_cow_to_mut(name: &str, was_borrowed: bool)" } },
    api11: { fns: { "track_thread_spawn": "pub fn track_thread_spawn<T>(name: &str, handle: JoinHandle<T>) -> JoinHandle<T>", "track_thread_join": "pub fn track_thread_join<T>(name: &str, result: thread::Result<T>) -> thread::Result<T>", "track_channel": "pub fn track_channel<T>(name: &str, tx: Sender<T>, rx: Receiver<T>) -> (Sender<T>, Receiver<T>)", "track_channel_send": "pub fn track_channel_send<T>(name: &str, result: Result<(), SendError<T>>) -> Result<(), SendError<T>>", "track_channel_recv": "pub fn track_channel_recv<T>(name: &str, result: Result<T, RecvError>) -> Result<T, RecvError>", "track_channel_try_recv": "pub fn track_channel_try_recv<T>(name: &str, result: Result<T, TryRecvError>) -> Result<T, TryRecvError>" } },
    api12: { fns: { "track_lock_guard_acquire": "pub fn track_lock_guard_acquire<T>(name: &str, lock_type: &str, guard: T) -> T", "track_lock_guard_drop": "pub fn track_lock_guard_drop(name: &str)" } },
    api13: { fns: { "track_once_cell_new": "pub fn track_once_cell_new<T>(name: &str, cell: OnceCell<T>) -> OnceCell<T>", "track_once_cell_set": "pub fn track_once_cell_set<T>(name: &str, result: Result<(), T>) -> Result<(), T>", "track_once_cell_get": "pub fn track_once_cell_get<'a, T>(name: &str, value: Option<&'a T>) -> Option<&'a T>", "track_once_cell_get_or_init": "pub fn track_once_cell_get_or_init<'a, T>(name: &str, value: &'a T, was_init: bool) -> &'a T", "track_once_lock_new": "pub fn track_once_lock_new<T>(name: &str, lock: OnceLock<T>) -> OnceLock<T>" } },
    api14: { fns: { "track_maybe_uninit_new": "pub fn track_maybe_uninit_new<T>(name: &str, uninit: MaybeUninit<T>) -> MaybeUninit<T>", "track_maybe_uninit_uninit": "pub fn track_maybe_uninit_uninit<T>(name: &str, uninit: MaybeUninit<T>) -> MaybeUninit<T>", "track_maybe_uninit_write": "pub fn track_maybe_uninit_write<'a, T>(name: &str, ptr: &'a mut T) -> &'a mut T", "track_maybe_uninit_assume_init": "pub unsafe fn track_maybe_uninit_assume_init<T>(name: &str, new_name: &str, value: T) -> T", "track_maybe_uninit_assume_init_read": "pub unsafe fn track_maybe_uninit_assume_init_read<T>(name: &str, value: T) -> T", "track_maybe_uninit_assume_init_drop": "pub unsafe fn track_maybe_uninit_assume_init_drop(name: &str)" } },
    api15: { fns: { "track_loop_enter": "pub fn track_loop_enter(loop_id: &str, loop_type: &str)", "track_loop_iteration": "pub fn track_loop_iteration(loop_id: &str, iteration: u64)", "track_loop_exit": "pub fn track_loop_exit(loop_id: &str)", "track_match_enter": "pub fn track_match_enter(match_id: &str)", "track_match_arm": "pub fn track_match_arm(match_id: &str, arm_index: usize)", "track_match_exit": "pub fn track_match_exit(match_id: &str)", "track_branch": "pub fn track_branch(branch_id: &str, condition: bool)", "track_return": "pub fn track_return(fn_id: &str)", "track_try": "pub fn track_try(expr_id: &str, success: bool)", "track_index_access": "pub fn track_index_access(container: &str, index: &str)", "track_field_access": "pub fn track_field_access(struct_name: &str, field: &str)", "track_call": "pub fn track_call(fn_name: &str, call_id: &str)", "track_lock": "pub fn track_lock(lock_id: &str, lock_type: &str)", "track_unwrap": "pub fn track_unwrap(expr_id: &str, success: bool)", "track_clone": "pub fn track_clone(source_id: &str, clone_id: &str)", "track_deref": "pub fn track_deref(ref_id: &str)", "track_break": "pub fn track_break(loop_id: &str)", "track_continue": "pub fn track_continue(loop_id: &str)", "track_closure_create": "pub fn track_closure_create(closure_id: &str, capture_count: usize)", "track_closure_capture": "pub fn track_closure_capture(closure_id: &str, var_id: &str, capture_type: &str)", "track_struct_create": "pub fn track_struct_create(name: &str, type_name: &str)", "track_tuple_create": "pub fn track_tuple_create(name: &str, element_count: usize)", "track_let_else": "pub fn track_let_else(var_id: &str, matched: bool)", "track_range": "pub fn track_range(range_id: &str, range_type: &str)", "track_binary_op": "pub fn track_binary_op(op: &str, lhs_id: &str, rhs_id: &str)", "track_array_create": "pub fn track_array_create(name: &str, length: usize)", "track_type_cast": "pub fn track_type_cast(expr_id: &str, from_type: &str, to_type: &str)", "track_region_enter": "pub fn track_region_enter(region_id: &str)", "track_region_exit": "pub fn track_region_exit(region_id: &str)", "track_fn_enter": "pub fn track_fn_enter(fn_name: &str, fn_id: &str)", "track_fn_exit": "pub fn track_fn_exit(fn_id: &str)" } }
};

window.updateApiDropdown = function(cat) {
    var fnSelect = document.getElementById('api-fn-select');
    var apiDetails = document.getElementById('api-details');
    if (!fnSelect) return;
    
    fnSelect.innerHTML = '<option value="">-- Select Function --</option>';
    if (apiDetails) apiDetails.style.display = 'none';
    
    if (cat && window.apiData[cat]) {
        fnSelect.removeAttribute('disabled');
        var fns = Object.keys(window.apiData[cat].fns).sort();
        for (var i = 0; i < fns.length; i++) {
            var opt = document.createElement('option');
            opt.value = fns[i];
            opt.textContent = fns[i];
            fnSelect.appendChild(opt);
        }
    } else {
        fnSelect.setAttribute('disabled', 'disabled');
    }
};

window.showApiDetails = function() {
    var catSelect = document.getElementById('api-category-select');
    var fnSelect = document.getElementById('api-fn-select');
    var apiDetails = document.getElementById('api-details');
    var apiCode = document.getElementById('api-code');
    if (!catSelect || !fnSelect || !apiDetails || !apiCode) return;
    
    var cat = catSelect.value;
    var fn = fnSelect.value;
    
    if (cat && fn && window.apiData[cat] && window.apiData[cat].fns[fn]) {
        var sig = window.apiData[cat].fns[fn];
        var pseudo = window.getApiPseudo(fn);
        
        // Syntax highlight the signature
        var highlighted = sig.replace(/\bpub\b/g, '<span class="rust-keyword">pub</span>')
                 .replace(/\bunsafe\b/g, '<span class="rust-keyword">unsafe</span>')
                 .replace(/\bfn\b/g, '<span class="rust-keyword">fn</span>')
                 .replace(/(&amp;'a mut |&'a mut )/g, '&\'a <span class="rust-keyword">mut</span> ')
                 .replace(/(&amp;mut |&mut )/g, '&<span class="rust-keyword">mut</span> ')
                 .replace(/(&amp;str|&str)/g, '&<span class="rust-keyword">str</span>')
                 .replace(/(track_\w+)/g, '<span class="rust-function">$1</span>')
                 .replace(/\b(Rc|Arc|Box|Pin|Cow|Weak|RefCell|Cell|Ref|RefMut|OnceCell|OnceLock|MaybeUninit|JoinHandle|Sender|Receiver|Option|Result|SendError|RecvError|TryRecvError)\b/g, '<span class="rust-type">$1</span>')
                 .replace(/\b(bool|usize|u64)\b/g, '<span class="rust-type">$1</span>');
        
        apiCode.innerHTML = highlighted + '\n\n<span class="rust-comment">// Implementation:</span>\n' + pseudo;
        apiDetails.style.display = 'block';
    } else {
        apiDetails.style.display = 'none';
    }
};

window.getApiPseudo = function(fn) {
    var codeMap = {
        // Basic Ownership
        "track_new": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> type_name = std::any::type_name::&lt;T&gt;();\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_new(name, type_name);\n}\nvalue',
        "track_borrow": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_borrow(name, owner_id, <span class="rust-keyword">false</span>);\n}\nvalue',
        "track_borrow_mut": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_borrow(name, owner_id, <span class="rust-keyword">true</span>);\n}\nvalue',
        "track_move": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_move(from_id, name);\n}\nvalue',
        "track_drop": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_drop(name);\n}',
        "track_drop_batch": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    <span class="rust-keyword">for</span> name <span class="rust-keyword">in</span> names {\n        tracker.record_drop(name);\n    }\n}',
        // Smart Pointers
        "track_rc_new": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> strong = <span class="rust-type">Rc</span>::strong_count(&value);\n    <span class="rust-keyword">let</span> weak = <span class="rust-type">Rc</span>::weak_count(&value);\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_rc_new(name, strong, weak);\n}\nvalue',
        "track_rc_clone": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> strong = <span class="rust-type">Rc</span>::strong_count(&value);\n    <span class="rust-keyword">let</span> weak = <span class="rust-type">Rc</span>::weak_count(&value);\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_rc_clone(name, source, strong, weak);\n}\nvalue',
        "track_arc_new": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> strong = <span class="rust-type">Arc</span>::strong_count(&value);\n    <span class="rust-keyword">let</span> weak = <span class="rust-type">Arc</span>::weak_count(&value);\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_arc_new(name, strong, weak);\n}\nvalue',
        "track_arc_clone": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> strong = <span class="rust-type">Arc</span>::strong_count(&value);\n    <span class="rust-keyword">let</span> weak = <span class="rust-type">Arc</span>::weak_count(&value);\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_arc_clone(name, source, strong, weak);\n}\nvalue',
        // Interior Mutability
        "track_refcell_new": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> type_name = std::any::type_name::&lt;T&gt;();\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_refcell_new(name, type_name);\n}\nvalue',
        "track_refcell_borrow": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_refcell_borrow(name, <span class="rust-keyword">false</span>);\n}\nguard',
        "track_refcell_borrow_mut": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_refcell_borrow(name, <span class="rust-keyword">true</span>);\n}\nguard',
        "track_cell_new": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> type_name = std::any::type_name::&lt;T&gt;();\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_cell_new(name, type_name);\n}\nvalue',
        "track_cell_get": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_cell_get(name);\n}\nvalue',
        "track_cell_set": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_cell_set(name);\n}',
        // Unsafe
        "track_raw_ptr": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> addr = ptr <span class="rust-keyword">as</span> <span class="rust-type">usize</span>;\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_raw_ptr(name, addr, <span class="rust-keyword">false</span>);\n}\nptr',
        "track_raw_ptr_mut": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> addr = ptr <span class="rust-keyword">as</span> <span class="rust-type">usize</span>;\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_raw_ptr(name, addr, <span class="rust-keyword">true</span>);\n}\nptr',
        "track_unsafe_block_enter": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_unsafe_block_enter(block_id);\n}',
        "track_unsafe_block_exit": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_unsafe_block_exit(block_id);\n}',
        "track_ffi_call": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_ffi_call(fn_name);\n}',
        "track_transmute": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_transmute(from_type, to_type);\n}',
        // Box/Weak/Pin/Cow
        "track_box_new": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> type_name = std::any::type_name::&lt;T&gt;();\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_box_new(name, type_name);\n}\nvalue',
        "track_weak_new": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_weak_new(name, source);\n}\nweak',
        "track_weak_upgrade": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> success = value.is_some();\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_weak_upgrade(name, source, success);\n}\nvalue',
        "track_pin_new": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> type_name = std::any::type_name::&lt;P&gt;();\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_pin_new(name, type_name);\n}\npin',
        "track_cow_borrowed": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> type_name = std::any::type_name::&lt;B&gt;();\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_cow_borrowed(name, type_name);\n}\ncow',
        "track_cow_to_mut": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_cow_to_mut(name, was_borrowed);\n}',
        // Thread/Channel
        "track_thread_spawn": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> thread_id = handle.thread().id();\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_thread_spawn(name, thread_id);\n}\nhandle',
        "track_channel": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_channel_new(name);\n}\n(tx, rx)',
        "track_channel_send": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> success = result.is_ok();\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_channel_send(name, success);\n}\nresult',
        // Lock Guards
        "track_lock_guard_acquire": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_lock_guard_acquire(name, lock_type);\n}\nguard',
        "track_lock_guard_drop": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_lock_guard_drop(name);\n}',
        // OnceCell/MaybeUninit
        "track_once_cell_new": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> type_name = std::any::type_name::&lt;T&gt;();\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_once_cell_new(name, type_name, <span class="rust-keyword">false</span>);\n}\ncell',
        "track_once_cell_get_or_init": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_once_cell_get_or_init(name, was_init);\n}\nvalue',
        "track_maybe_uninit_new": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> type_name = std::any::type_name::&lt;T&gt;();\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_maybe_uninit_new(name, type_name, <span class="rust-keyword">true</span>);\n}\nuninit',
        "track_maybe_uninit_assume_init": '#[cfg(feature = "track")]\n<span class="rust-keyword">unsafe</span> {\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_maybe_uninit_assume_init(name, new_name);\n}\nvalue',
        // Control Flow
        "track_fn_enter": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_fn_enter(fn_name, fn_id);\n}',
        "track_fn_exit": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_fn_exit(fn_id);\n}',
        "track_loop_enter": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_loop_enter(loop_id, loop_type);\n}',
        "track_closure_create": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_closure_create(closure_id, capture_count);\n}',
        "track_closure_capture": '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_closure_capture(closure_id, var_id, capture_type);\n}'
    };
    // Generate generic implementation for unmapped functions
    var eventName = fn.replace('track_', '').replace(/_([a-z])/g, function(m, c) { return c.toUpperCase(); });
    eventName = eventName.charAt(0).toUpperCase() + eventName.slice(1);
    return codeMap[fn] || '#[cfg(feature = "track")]\n{\n    <span class="rust-keyword">let</span> <span class="rust-keyword">mut</span> tracker = TRACKER.lock();\n    tracker.record_' + fn.replace('track_', '') + '(...);\n}\nvalue';
};


// Collapsible ToC
function toggleToc(element) {
    var subsections = element.parentElement.querySelector('.toc-subsections');
    if (subsections) {
        subsections.classList.toggle('collapsed');
        element.textContent = subsections.classList.contains('collapsed') ? '▶' : '▼';
    }
}
