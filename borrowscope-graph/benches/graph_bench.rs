use borrowscope_graph::conflict::find_conflicts;
use borrowscope_graph::export::{to_json, to_json_compact, to_msgpack};
use borrowscope_graph::stats::statistics;
use borrowscope_graph::traversal::*;
use borrowscope_graph::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn build_chain_graph(n: usize) -> OwnershipGraph {
    let mut g = OwnershipGraph::new();
    let mut nodes = Vec::new();
    for i in 0..n {
        nodes.push(g.add_variable(&format!("v{}", i), "i32", i as u64));
    }
    for i in 1..n {
        g.add_borrow(nodes[i], nodes[i - 1], false, i as u64);
    }
    g
}

fn build_star_graph(n: usize) -> OwnershipGraph {
    let mut g = OwnershipGraph::new();
    let center = g.add_variable("center", "Vec<i32>", 0);
    for i in 0..n {
        let r = g.add_variable(&format!("r{}", i), "&Vec<i32>", (i + 1) as u64);
        let eid = g.add_borrow(r, center, false, (i + 1) as u64);
        g.end_edge(eid, (i + 1) as u64 + 5);
    }
    g
}

fn bench_from_events(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_events");
    for size in [100, 1_000, 10_000] {
        let events: Vec<_> = (0..size)
            .map(|i| borrowscope_runtime::Event::New {
                timestamp: i as u64,
                var_name: format!("v{}", i),
                var_id: format!("v{}_0", i),
                type_name: "i32".to_string(),
            })
            .collect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &events, |b, events| {
            b.iter(|| OwnershipGraph::from_events(events))
        });
    }
    group.finish();
}

fn bench_dfs(c: &mut Criterion) {
    let mut group = c.benchmark_group("dfs");
    for size in [100, 1_000, 10_000] {
        let graph = build_chain_graph(size);
        let start = NodeId(0);
        group.bench_with_input(BenchmarkId::from_parameter(size), &graph, |b, graph| {
            b.iter(|| dfs(graph, start, Direction::Outgoing))
        });
    }
    group.finish();
}

fn bench_conflict_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_conflicts");
    for size in [100, 500, 1_000] {
        let graph = build_star_graph(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &graph, |b, graph| {
            b.iter(|| find_conflicts(graph))
        });
    }
    group.finish();
}

fn bench_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");
    for size in [100, 1_000, 10_000] {
        let graph = build_chain_graph(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &graph, |b, graph| {
            b.iter(|| statistics(graph))
        });
    }
    group.finish();
}

fn bench_json_export(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_export");
    for size in [100, 1_000] {
        let graph = build_chain_graph(size);
        group.bench_with_input(BenchmarkId::new("full", size), &graph, |b, graph| {
            b.iter(|| to_json(graph).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("compact", size), &graph, |b, graph| {
            b.iter(|| to_json_compact(graph).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("msgpack", size), &graph, |b, graph| {
            b.iter(|| to_msgpack(graph).unwrap())
        });
    }
    group.finish();
}

fn bench_connected_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("connected_components");
    for size in [100, 1_000, 10_000] {
        let graph = build_chain_graph(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &graph, |b, graph| {
            b.iter(|| connected_components(graph))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_from_events,
    bench_dfs,
    bench_conflict_detection,
    bench_statistics,
    bench_json_export,
    bench_connected_components,
);
criterion_main!(benches);
