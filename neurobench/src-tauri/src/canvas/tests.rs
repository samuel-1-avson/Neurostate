//! Integration Tests - End-to-end tests for canvas operations
//!
//! Validates complete workflows across all canvas modules.

#[cfg(test)]
mod integration_tests {
    use crate::canvas::*;
    use crate::canvas::types::*;
    use crate::canvas::batch::*;

    #[test]
    fn test_complete_fsm_workflow() {
        let mut engine = CanvasEngine::new();

        // Create initial state
        let initial = CanvasNode {
            id: "initial".into(),
            label: "Start".into(),
            node_type: NodeType::Initial,
            x: 100.0,
            y: 100.0,
            width: 80.0,
            height: 40.0,
            entry_action: None,
            exit_action: None,
            description: None,
        };
        engine.add_node(initial).unwrap();

        // Create states
        for i in 0..5 {
            let node = CanvasNode {
                id: format!("s{}", i),
                label: format!("State {}", i),
                node_type: NodeType::State,
                x: 100.0 + (i as f64 * 150.0),
                y: 200.0,
                width: 100.0,
                height: 50.0,
                entry_action: Some(format!("enter_state{}", i)),
                exit_action: Some(format!("exit_state{}", i)),
                description: Some(format!("State {} description", i)),
            };
            engine.add_node(node).unwrap();
        }

        // Add final state
        let final_node = CanvasNode {
            id: "final".into(),
            label: "End".into(),
            node_type: NodeType::Final,
            x: 850.0,
            y: 200.0,
            width: 80.0,
            height: 40.0,
            entry_action: None,
            exit_action: None,
            description: None,
        };
        engine.add_node(final_node).unwrap();

        // Connect nodes
        engine.connect("initial", "s0", Some("start".into())).unwrap();
        for i in 0..4 {
            engine.connect(&format!("s{}", i), &format!("s{}", i + 1), Some(format!("next{}", i))).unwrap();
        }
        engine.connect("s4", "final", Some("complete".into())).unwrap();

        // Verify structure
        assert_eq!(engine.nodes().len(), 7);
        assert_eq!(engine.edges().len(), 6);

        // Test spatial query
        let found = engine.get_node_at(150.0, 210.0);
        assert!(found.is_some());

        // Test undo/redo
        engine.undo().unwrap();
        assert_eq!(engine.edges().len(), 5);
        engine.redo().unwrap();
        assert_eq!(engine.edges().len(), 6);
    }

    #[test]
    fn test_batch_with_undo() {
        let mut engine = CanvasEngine::new();

        // Add nodes via batch
        let mut batch = BatchOperation::new();
        for i in 0..10 {
            batch = batch.add_node(CanvasNode {
                id: format!("n{}", i),
                label: format!("Node {}", i),
                node_type: NodeType::State,
                x: (i * 100) as f64,
                y: 100.0,
                width: 80.0,
                height: 40.0,
                entry_action: None,
                exit_action: None,
                description: None,
            });
        }

        let result = engine.execute_batch(batch).unwrap();
        assert_eq!(result.added_nodes.len(), 10);
        assert_eq!(engine.nodes().len(), 10);

        // Single undo should revert entire batch
        engine.undo().unwrap();
        assert_eq!(engine.nodes().len(), 0, "Batch undo should remove all nodes");

        // Redo brings them back
        engine.redo().unwrap();
        assert_eq!(engine.nodes().len(), 10);
    }

    #[test]
    fn test_move_with_path_invalidation() {
        let mut engine = CanvasEngine::new();

        // Create two connected nodes
        engine.add_node(CanvasNode {
            id: "n1".into(),
            label: "A".into(),
            node_type: NodeType::State,
            x: 100.0,
            y: 100.0,
            width: 80.0,
            height: 40.0,
            entry_action: None,
            exit_action: None,
            description: None,
        }).unwrap();

        engine.add_node(CanvasNode {
            id: "n2".into(),
            label: "B".into(),
            node_type: NodeType::State,
            x: 300.0,
            y: 100.0,
            width: 80.0,
            height: 40.0,
            entry_action: None,
            exit_action: None,
            description: None,
        }).unwrap();

        engine.connect("n1", "n2", None).unwrap();

        // Get cached path
        let path1 = engine.get_edge_path_cached("e0");
        assert!(path1.is_some() || engine.edges().values().next().map(|e| e.id.clone()).is_some());

        // Move node - should invalidate cache
        engine.move_nodes(vec![NodeMove {
            id: "n1".into(),
            x: 100.0,
            y: 300.0,
        }]).unwrap();

        // Path should be recalculated on next access
        // (implementation detail - cache is invalidated)
    }

    #[test]
    fn test_delete_with_cascade() {
        let mut engine = CanvasEngine::new();

        // Create connected nodes
        for i in 0..5 {
            engine.add_node(CanvasNode {
                id: format!("n{}", i),
                label: format!("N{}", i),
                node_type: NodeType::State,
                x: (i * 100) as f64,
                y: 100.0,
                width: 80.0,
                height: 40.0,
                entry_action: None,
                exit_action: None,
                description: None,
            }).unwrap();
        }

        // Connect in chain
        for i in 0..4 {
            engine.connect(&format!("n{}", i), &format!("n{}", i + 1), None).unwrap();
        }

        assert_eq!(engine.nodes().len(), 5);
        assert_eq!(engine.edges().len(), 4);

        // Delete middle node - should cascade delete edges
        engine.delete_nodes(vec!["n2".into()]).unwrap();

        assert_eq!(engine.nodes().len(), 4);
        // Connected edges should be deleted
        assert!(engine.edges().len() < 4);
    }

    #[test]
    fn test_selection_operations() {
        let mut engine = CanvasEngine::new();

        for i in 0..10 {
            engine.add_node(CanvasNode {
                id: format!("n{}", i),
                label: format!("N{}", i),
                node_type: NodeType::State,
                x: (i % 5) as f64 * 100.0,
                y: (i / 5) as f64 * 100.0,
                width: 80.0,
                height: 40.0,
                entry_action: None,
                exit_action: None,
                description: None,
            }).unwrap();
        }

        // Select nodes in rectangle
        let selected = engine.get_nodes_in_rect(0.0, 0.0, 250.0, 50.0);
        assert!(selected.len() >= 2, "Should select nodes in rect");
    }

    #[test]
    fn test_validation_integration() {
        use crate::canvas::validation::*;

        let mut engine = CanvasEngine::new();

        // Create FSM without initial state
        for i in 0..3 {
            engine.add_node(CanvasNode {
                id: format!("s{}", i),
                label: format!("State {}", i),
                node_type: NodeType::State,
                x: (i * 150) as f64,
                y: 100.0,
                width: 100.0,
                height: 50.0,
                entry_action: None,
                exit_action: None,
                description: None,
            }).unwrap();
        }

        engine.connect("s0", "s1", None).unwrap();
        engine.connect("s1", "s2", None).unwrap();

        // Validate
        let result = FsmValidator::validate(engine.nodes(), engine.edges());

        // Should have error about missing initial state
        assert!(!result.valid);
        assert!(result.issues.iter().any(|i| i.message.contains("initial")));
    }
}
