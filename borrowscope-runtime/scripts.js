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
