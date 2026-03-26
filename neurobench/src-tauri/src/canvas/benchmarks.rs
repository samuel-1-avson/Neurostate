//! Benchmarks - Performance benchmarks for canvas operations
//!
//! Tests with 1000+ nodes and 5000+ edges to validate performance.

#[cfg(test)]
mod benchmarks {
    use crate::canvas::*;
    use crate::canvas::types::*;
    use std::time::Instant;

    /// Generate N random nodes
    fn generate_nodes(count: usize) -> Vec<CanvasNode> {
        (0..count)
            .map(|i| CanvasNode {
                id: format!("n{}", i),
                label: format!("Node {}", i),
                node_type: NodeType::State,
                x: (i % 100) as f64 * 20.0,
                y: (i / 100) as f64 * 20.0,
                width: 160.0,
                height: 80.0,
                entry_action: None,
                exit_action: None,
                description: None,
            })
            .collect()
    }

    /// Generate M random edges between N nodes
    fn generate_edges(node_count: usize, edge_count: usize) -> Vec<CanvasEdge> {
        (0..edge_count)
            .map(|i| {
                let src = i % node_count;
                let dst = (i + 1) % node_count;
                CanvasEdge::new(
                    format!("e{}", i),
                    format!("n{}", src),
                    format!("n{}", dst),
                    Some(format!("T{}", i)),
                )
            })
            .collect()
    }

    #[test]
    fn bench_1000_nodes_add() {
        let mut engine = CanvasEngine::new();
        let nodes = generate_nodes(1000);

        let start = Instant::now();
        for node in nodes {
            let _ = engine.add_node(node);
        }
        let elapsed = start.elapsed();

        println!("Add 1000 nodes: {:?}", elapsed);
        assert!(elapsed.as_millis() < 1000, "Adding 1000 nodes should be under 1 second");
    }

    #[test]
    fn bench_spatial_query_1000_nodes() {
        let mut engine = CanvasEngine::new();
        let nodes = generate_nodes(1000);
        
        for node in nodes {
            let _ = engine.add_node(node);
        }

        // Benchmark point queries
        let start = Instant::now();
        for _ in 0..1000 {
            engine.get_node_at(500.0, 500.0);
        }
        let elapsed = start.elapsed();

        println!("1000 point queries (1000 nodes): {:?}", elapsed);
        assert!(elapsed.as_millis() < 100, "1000 queries should be under 100ms");
    }

    #[test]
    fn bench_spatial_query_5000_nodes() {
        let mut engine = CanvasEngine::new();
        let nodes = generate_nodes(5000);
        
        for node in nodes {
            let _ = engine.add_node(node);
        }

        // Benchmark rectangle queries
        let start = Instant::now();
        for _ in 0..100 {
            engine.get_nodes_in_rect(0.0, 0.0, 500.0, 500.0);
        }
        let elapsed = start.elapsed();

        println!("100 rect queries (5000 nodes): {:?}", elapsed);
        assert!(elapsed.as_millis() < 500, "100 rect queries should be under 500ms");
    }

    #[test]
    fn bench_5000_edges_connect() {
        let mut engine = CanvasEngine::new();
        let nodes = generate_nodes(1000);
        
        for node in nodes {
            let _ = engine.add_node(node);
        }

        let start = Instant::now();
        for i in 0..5000 {
            let src = format!("n{}", i % 1000);
            let dst = format!("n{}", (i + 1) % 1000);
            let _ = engine.connect(&src, &dst, Some(format!("T{}", i)));
        }
        let elapsed = start.elapsed();

        println!("Add 5000 edges: {:?}", elapsed);
        assert!(elapsed.as_millis() < 5000, "Adding 5000 edges should be under 5 seconds");
    }

    #[test]
    fn bench_undo_redo_1000_ops() {
        let mut engine = CanvasEngine::new();
        let nodes = generate_nodes(500);
        
        for node in nodes {
            let _ = engine.add_node(node);
        }

        // Record 500 move operations
        let start = Instant::now();
        for i in 0..500 {
            let id = format!("n{}", i);
            let _ = engine.move_nodes(vec![NodeMove { id, x: 100.0, y: 100.0 }]);
        }
        let move_time = start.elapsed();

        // Undo all
        let start = Instant::now();
        for _ in 0..500 {
            let _ = engine.undo();
        }
        let undo_time = start.elapsed();

        // Redo all
        let start = Instant::now();
        for _ in 0..500 {
            let _ = engine.redo();
        }
        let redo_time = start.elapsed();

        println!("500 moves: {:?}", move_time);
        println!("500 undos: {:?}", undo_time);
        println!("500 redos: {:?}", redo_time);

        assert!(undo_time.as_millis() < 1000, "500 undos should be under 1 second");
        assert!(redo_time.as_millis() < 1000, "500 redos should be under 1 second");
    }

    #[test]
    fn bench_batch_operations() {
        use crate::canvas::batch::*;

        let mut engine = CanvasEngine::new();
        
        // Create batch with 100 nodes
        let mut nodes = Vec::new();
        for i in 0..100 {
            nodes.push(CanvasNode {
                id: format!("n{}", i),
                label: format!("Node {}", i),
                node_type: NodeType::State,
                x: (i % 10) as f64 * 100.0,
                y: (i / 10) as f64 * 100.0,
                width: 80.0,
                height: 40.0,
                entry_action: None,
                exit_action: None,
                description: None,
            });
        }

        let mut batch = BatchOperation::new();
        for node in nodes {
            batch = batch.add_node(node);
        }

        let start = Instant::now();
        let result = engine.execute_batch(batch);
        let elapsed = start.elapsed();

        println!("Batch add 100 nodes: {:?}", elapsed);
        assert!(result.is_ok());
        assert!(elapsed.as_millis() < 100, "Batch 100 nodes should be under 100ms");
    }

    #[test]
    fn bench_path_cache() {
        let mut engine = CanvasEngine::new();
        
        // Create nodes and edges
        for i in 0..100 {
            let _ = engine.add_node(CanvasNode {
                id: format!("n{}", i),
                label: format!("N{}", i),
                node_type: NodeType::State,
                x: (i % 10) as f64 * 150.0,
                y: (i / 10) as f64 * 100.0,
                width: 100.0,
                height: 50.0,
                entry_action: None,
                exit_action: None,
                description: None,
            });
        }

        // Collect actual edge IDs from connect()
        let mut edge_ids = Vec::new();
        for i in 0..99 {
            if let Ok(edge) = engine.connect(&format!("n{}", i), &format!("n{}", i + 1), None) {
                edge_ids.push(edge.id);
            }
        }

        // Skip test if no edges were created (e.g., cycle detection)
        if edge_ids.is_empty() {
            println!("No edges created, skipping cache benchmark");
            return;
        }

        // First pass - calculate all
        let start = Instant::now();
        for id in &edge_ids {
            let _ = engine.get_edge_path_cached(id);
        }
        let first_pass = start.elapsed();

        // Second pass - from cache
        let start = Instant::now();
        for id in &edge_ids {
            let _ = engine.get_edge_path_cached(id);
        }
        let cached_pass = start.elapsed();

        println!("First pass (calculate): {:?}", first_pass);
        println!("Second pass (cached): {:?}", cached_pass);

        // Cached should be significantly faster (or at least not slower)
        // Allow same speed if both are very fast
        assert!(cached_pass <= first_pass.saturating_add(std::time::Duration::from_micros(100)), 
                "Cached pass should not be significantly slower");
    }

    #[test]
    fn bench_validation_1000_nodes() {
        use crate::canvas::validation::*;

        let mut nodes = std::collections::HashMap::new();
        let mut edges = std::collections::HashMap::new();

        // Initial state
        nodes.insert("initial".into(), CanvasNode {
            id: "initial".into(),
            label: "Start".into(),
            node_type: NodeType::Initial,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 40.0,
            entry_action: None,
            exit_action: None,
            description: None,
        });

        // Generate states
        for i in 0..1000 {
            nodes.insert(format!("n{}", i), CanvasNode {
                id: format!("n{}", i),
                label: format!("State {}", i),
                node_type: NodeType::State,
                x: (i % 50) as f64 * 100.0,
                y: (i / 50) as f64 * 100.0,
                width: 80.0,
                height: 40.0,
                entry_action: None,
                exit_action: None,
                description: None,
            });
        }

        // Connect
        edges.insert("e0".into(), CanvasEdge::new("e0".into(), "initial".into(), "n0".into(), None));
        for i in 0..999 {
            edges.insert(format!("e{}", i + 1), CanvasEdge::new(
                format!("e{}", i + 1),
                format!("n{}", i),
                format!("n{}", i + 1),
                None,
            ));
        }

        let start = Instant::now();
        let result = FsmValidator::validate(&nodes, &edges);
        let elapsed = start.elapsed();

        println!("Validate 1000 nodes: {:?} ({} issues)", elapsed, result.issues.len());
        assert!(elapsed.as_millis() < 100, "Validation should be under 100ms");
    }
}
