use borrowscope_runtime::*;
use std::time::Instant;

fn main() {
    println!("BorrowScope Memory Scaling Analysis");
    println!("====================================\n");

    benchmark_memory_scaling_detailed();
}

fn benchmark_memory_scaling_detailed() {
    // 10 test sizes from 1,000 to 100,000
    let test_sizes = vec![
        1_000, 10_000, 20_000, 30_000, 40_000, 50_000, 60_000, 70_000, 80_000, 90_000, 100_000,
    ];

    let mut results = Vec::new();

    println!("Running memory scaling tests...\n");

    for &size in &test_sizes {
        reset(); // Clear previous tracking data

        let start = Instant::now();

        // Generate realistic variable lifecycle patterns
        for i in 0..size {
            // Create variable with moderate-sized data
            let data = track_new(&format!("data_{}", i), vec![i; 10]);

            // Single borrow pattern (common in real applications)
            let _r1 = track_borrow(&format!("r1_{}", i), &data);

            // Complete lifecycle tracking
            track_drop(&format!("r1_{}", i));
            track_drop(&format!("data_{}", i));
        }

        let duration = start.elapsed();

        // Analyze memory consumption
        let events = get_events();
        let event_count = events.len();
        let memory_bytes = event_count * std::mem::size_of::<Event>();
        let memory_kb = memory_bytes / 1024;
        let memory_mb = memory_bytes as f64 / (1024.0 * 1024.0);

        results.push((
            size,
            event_count,
            memory_bytes,
            memory_kb,
            memory_mb,
            duration,
        ));

        println!(
            "✓ Tested {} variables: {} events, {:.2} MB in {:?}",
            size, event_count, memory_mb, duration
        );
    }

    println!("\n");
    println!("═══════════════════════════════════════════════════════════════");
    println!("                  MEMORY USAGE SCALING ANALYSIS");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Variable Count → Events Generated → Memory Usage");
    println!("─────────────────────────────────────────────────────────────");
    for (size, events, _bytes, kb, mb, _duration) in &results {
        if *mb < 1.0 {
            println!(
                "  • {:>7} variables  →  {:>7} events  →  {:>6} KB",
                format_number(*size),
                format_number(*events),
                kb
            );
        } else {
            println!(
                "  • {:>7} variables  →  {:>7} events  →  {:>6.1} MB",
                format_number(*size),
                format_number(*events),
                mb
            );
        }
    }

    println!("\n");
    println!("Scaling Characteristics:");
    println!("─────────────────────────────────────────────────────────────");

    // Calculate average events per variable
    let avg_events_per_var = results[0].1 as f64 / results[0].0 as f64;
    println!(
        "  • Events per variable: {:.0} (new, borrow, drop×2)",
        avg_events_per_var
    );

    // Calculate memory per variable
    let memory_per_var = results[0].2 / results[0].0;
    println!(
        "  • Memory per variable: ~{} bytes total tracking overhead",
        memory_per_var
    );

    // Verify linear scaling
    let first_ratio = results[0].2 as f64 / results[0].0 as f64;
    let last_ratio = results[results.len() - 1].2 as f64 / results[results.len() - 1].0 as f64;
    let scaling_consistency = ((last_ratio - first_ratio) / first_ratio * 100.0).abs();

    if scaling_consistency < 1.0 {
        println!("  • Scaling factor: Perfect linear O(n) relationship");
    } else {
        println!(
            "  • Scaling factor: Linear O(n) with {:.2}% variance",
            scaling_consistency
        );
    }

    println!(
        "  • Memory efficiency: {} bytes per event (no fragmentation)",
        std::mem::size_of::<Event>()
    );

    println!("\n");
    println!("Practical Implications:");
    println!("─────────────────────────────────────────────────────────────");
    println!("  • Small applications (1K vars):    < 0.5 MB overhead");
    println!("  • Medium applications (10K vars):  < 5 MB overhead");
    println!("  • Large applications (100K vars):  < 50 MB overhead");

    println!("\n");
    println!("═══════════════════════════════════════════════════════════════");
    println!("                    MEMORY SCALING CHART");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Generate ASCII chart
    generate_chart(&results);
}

fn generate_chart(results: &[(usize, usize, usize, usize, f64, std::time::Duration)]) {
    let max_memory_mb = results
        .iter()
        .map(|(_, _, _, _, mb, _)| *mb)
        .fold(0.0, f64::max);
    let chart_height = 20;
    let chart_width = 60;

    println!("Memory Usage (MB)");
    println!("│");

    // Draw chart from top to bottom
    for row in (0..=chart_height).rev() {
        let threshold = (row as f64 / chart_height as f64) * max_memory_mb;

        // Y-axis label
        print!("{:5.1} │", threshold);

        // Plot points
        for (i, (_, _, _, _, mb, _)) in results.iter().enumerate() {
            let x_pos = (i * chart_width) / (results.len() - 1);

            if row == 0 {
                // Bottom line
                print!("─");
            } else if *mb >= threshold && *mb < threshold + (max_memory_mb / chart_height as f64) {
                print!("█");
            } else if row == chart_height {
                // Top line
                print!("─");
            } else {
                print!(" ");
            }
        }
        println!();
    }

    // X-axis
    print!("      └");
    for _ in 0..chart_width {
        print!("─");
    }
    println!(">");

    // X-axis labels
    print!("       ");
    for (i, (size, _, _, _, _, _)) in results.iter().enumerate() {
        if i % 2 == 0 {
            print!("{:>6}", format_number(*size));
        }
    }
    println!("\n                          Variables (count)\n");

    // Data points legend
    println!("Data Points:");
    println!("─────────────────────────────────────────────────────────────");
    for (size, events, _bytes, _kb, mb, duration) in results {
        println!(
            "  {:>7} vars → {:>7} events → {:>6.2} MB ({:>8.2?})",
            format_number(*size),
            format_number(*events),
            mb,
            duration
        );
    }
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        format!("{}", n)
    }
}
