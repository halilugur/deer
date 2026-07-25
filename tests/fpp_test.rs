#[cfg(test)]
mod tests {
    use deer::engine::evaluator::{Evaluator, Value};
    use deer::engine::runner::{ExecutionState, Runner};
    use deer::model::diagram::Diagram;
    use deer::model::node::NodeType;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_evaluator_basic() {
        let mut vars = HashMap::new();
        vars.insert("a".to_string(), Value::Number(5.0));

        let res = Evaluator::parse_value("a + 10", &vars);
        assert_eq!(res.to_number(), 15.0);

        let eq_res = Evaluator::eval_equal("a", "5", &vars);
        assert!(eq_res);

        let neq_res = Evaluator::eval_equal("a", "10", &vars);
        assert!(!neq_res);
    }

    #[test]
    fn test_gui_property_and_uninitialized_var_resolution() {
        let mut vars = HashMap::new();
        // 1. Plain unquoted text evaluates directly as literal string without needing "" quotes
        let res_tb = Evaluator::parse_value("x", &vars);
        assert_eq!(res_tb.to_string_val(), "x");

        let res_var = Evaluator::parse_value("my_custom_var", &vars);
        assert_eq!(res_var.to_string_val(), "my_custom_var");

        // 2. Setting variable in vars resolves variable name over literal
        vars.insert("x".to_string(), Value::String("Hello User".to_string()));
        let res_prop = Evaluator::parse_value("x", &vars);
        assert_eq!(res_prop.to_string_val(), "Hello User");

        let res_case = Evaluator::parse_value("X", &vars);
        assert_eq!(res_case.to_string_val(), "Hello User");
    }

    #[test]
    fn test_parse_sample_fpp() {
        let sample = r#"
3       	 <--SHAPES
2       	 <--LINES
id1
2       	 <--TYPE
298       	 <--LEFT
78       	 <--TOP
70       	 <--WIDTH
30       	 <--HEIGHT
16777215       	 <--BACKCOLOR
0       	 <--BORDERCOLOR
0       	 <--BORDERCOLOR
-reserved 1-
-reserved 2-
START

id2
91       	 <--TYPE
267       	 <--LEFT
149       	 <--TOP
132       	 <--WIDTH
40       	 <--HEIGHT
16777215       	 <--BACKCOLOR
0       	 <--BORDERCOLOR
0       	 <--BORDERCOLOR
-reserved 1-
-reserved 2-
OUTPUT
Hello World!

id3
2       	 <--TYPE
297       	 <--LEFT
234       	 <--TOP
70       	 <--WIDTH
30       	 <--HEIGHT
16777215       	 <--BACKCOLOR
0       	 <--BORDERCOLOR
0       	 <--BORDERCOLOR
-reserved 1-
-reserved 2-
STOP

  
---- LINES ---- from,to ----
id1,id2
reserved 1

id2,id3
reserved 1
"#;

        let diagram = Diagram::parse_fpp(sample).expect("Should parse FPP format successfully");
        assert_eq!(diagram.nodes.len(), 3);
        assert_eq!(diagram.connectors.len(), 2);
        assert_eq!(diagram.nodes[0].node_type, NodeType::Start);
        assert_eq!(diagram.nodes[1].node_type, NodeType::Output);
        assert_eq!(diagram.nodes[2].node_type, NodeType::Stop);
    }

    #[test]
    fn test_parse_all_support_files() {
        let support_dir = Path::new("Support");
        if !support_dir.exists() {
            return;
        }

        let entries = fs::read_dir(support_dir).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("fpp") {
                let content = fs::read_to_string(&path).unwrap();
                let diagram = Diagram::parse_fpp(&content);
                assert!(
                    diagram.is_ok(),
                    "Failed to parse legacy .fpp file: {:?}",
                    path
                );
                let diag = diagram.unwrap();
                assert!(!diag.nodes.is_empty(), "Nodes should not be empty for {:?}", path);

                // Round-trip export test
                let exported = diag.export_fpp();
                let re_parsed = Diagram::parse_fpp(&exported).unwrap();
                assert_eq!(
                    diag.nodes.len(),
                    re_parsed.nodes.len(),
                    "Roundtrip node count mismatch for {:?}",
                    path
                );
            }
        }
    }

    #[test]
    fn test_function_subdiagram_execution() {
        let call_path = Path::new("example/test10_call.fpp");
        if !call_path.exists() {
            return;
        }

        let content = fs::read_to_string(call_path).unwrap();
        let parent_diagram = Diagram::parse_fpp(&content).unwrap();

        let mut runner = Runner::new();
        runner.start(&parent_diagram);

        // Execute until finished or max 50 steps
        let mut steps = 0;
        while runner.state == ExecutionState::Running && steps < 50 {
            runner.step(&parent_diagram);
            steps += 1;
        }

        assert_eq!(runner.state, ExecutionState::Finished);
        let ret_a = runner.variables.get("a").map(|v| v.to_number()).unwrap_or(0.0);
        assert_eq!(ret_a, 123.0, "Variable `a` should receive returned value 123 from function.fpp");
    }

    #[test]
    fn test_factorial_fpp_execution() {
        let fact_path = Path::new("example/factorial.fpp");
        if !fact_path.exists() {
            return;
        }

        let content = fs::read_to_string(fact_path).unwrap();
        let diagram = Diagram::parse_fpp(&content).unwrap();

        let mut runner = Runner::new();
        runner.start(&diagram);

        // Step to INPUT node
        while runner.state == ExecutionState::Running {
            runner.step(&diagram);
        }

        // Submit input "5" for n
        if let ExecutionState::WaitingForInput { prompt: _, target_var, next_node_id } = runner.state.clone() {
            assert_eq!(target_var, "n");
            runner.submit_input("5", &target_var, &next_node_id);
        }

        // Step to next input or finish
        let mut steps = 0;
        while runner.state == ExecutionState::Running && steps < 30 {
            runner.step(&diagram);
            steps += 1;
        }

        let b_val = runner.variables.get("b").map(|v| v.to_number()).unwrap_or(0.0);
        assert_eq!(b_val, 120.0, "Factorial of 5 should equal 120.0");
    }

    #[test]
    fn test_b4_fpp_execution() {
        let b4_path = Path::new("examples/B4.fpp");
        if !b4_path.exists() {
            return;
        }

        let content = fs::read_to_string(b4_path).unwrap();
        let diagram = Diagram::parse_fpp(&content).unwrap();

        let mut runner = Runner::new();
        runner.start(&diagram);

        let mut steps = 0;
        while runner.state == ExecutionState::Running && steps < 20 {
            runner.step(&diagram);
            steps += 1;
        }

        assert_eq!(runner.state, ExecutionState::Finished);
        let a_val = runner.variables.get("a").map(|v| v.to_string_val()).unwrap_or_default();
        assert_eq!(a_val, "Merhaba Dünya!", "Variable a should contain Merhaba Dünya!");
    }

    #[test]
    fn test_step_single_debugging() {
        let b4_path = Path::new("examples/B4.fpp");
        if !b4_path.exists() {
            return;
        }

        let content = fs::read_to_string(b4_path).unwrap();
        let diagram = Diagram::parse_fpp(&content).unwrap();

        let mut runner = Runner::new();
        assert_eq!(runner.state, ExecutionState::Idle);

        // Step 1: START -> Paused at DEFINITION node
        runner.step_single(&diagram);
        assert_eq!(runner.state, ExecutionState::Paused);

        // Step 2: DEFINITION -> Paused at OUTPUT node
        runner.step_single(&diagram);
        assert_eq!(runner.state, ExecutionState::Paused);

        // Step 3: OUTPUT -> Paused at STOP node
        runner.step_single(&diagram);
        assert_eq!(runner.state, ExecutionState::Paused);

        // Step 4: STOP -> Finished
        runner.step_single(&diagram);
        assert_eq!(runner.state, ExecutionState::Finished);
    }

    #[test]
    fn test_step_single_with_input_dialog() {
        let fact_path = Path::new("example/factorial.fpp");
        if !fact_path.exists() {
            return;
        }

        let content = fs::read_to_string(fact_path).unwrap();
        let diagram = Diagram::parse_fpp(&content).unwrap();

        let mut runner = Runner::new();
        assert_eq!(runner.state, ExecutionState::Idle);

        // Step 1: Start node -> transitions to Input node (WaitingForInput)
        runner.step_single(&diagram);
        assert!(matches!(runner.state, ExecutionState::WaitingForInput { .. }));

        // Provide input text "5" into runner.input_text
        runner.input_text = "5".to_string();

        // Step 2: Calling step_single while in WaitingForInput submits input and pauses at next node
        runner.step_single(&diagram);
        assert_eq!(runner.state, ExecutionState::Paused);
        let n_val = runner.variables.get("n").map(|v| v.to_number()).unwrap_or(0.0);
        assert_eq!(n_val, 5.0);

        // Step 3: Continues stepping in Paused state
        runner.step_single(&diagram);
        assert_eq!(runner.state, ExecutionState::Paused);
    }

    #[test]
    fn test_input_text_exit_comparison() {
        let mut runner = Runner::new();
        runner.submit_input("exit", "num", "next_id");
        let num_val = runner.variables.get("num").map(|v| v.to_string_val()).unwrap_or_default();
        assert_eq!(num_val, "exit");

        let is_eq = Evaluator::eval_equal("num", "exit", &runner.variables);
        assert!(is_eq, "num holding 'exit' must equal exit without quotes");
    }

    #[test]
    fn test_modern_dfpp_json_diagram() {
        let dfpp_path = Path::new("examples/factorial.dfpp");
        assert!(dfpp_path.exists());

        let content = fs::read_to_string(dfpp_path).unwrap();
        let diagram = Diagram::parse(&content).expect("Should parse modern .dfpp JSON diagram");

        assert_eq!(diagram.nodes.len(), 8);
        assert_eq!(diagram.connectors.len(), 8);

        let exported_json = diagram.export_json().expect("Should export clean JSON");
        let re_parsed = Diagram::parse(&exported_json).expect("Should re-parse exported JSON");
        assert_eq!(re_parsed.nodes.len(), 8);
    }

    #[test]
    fn test_trig_calc_execution() {
        let trig_path = Path::new("examples/trig_calc.dfpp");
        assert!(trig_path.exists());

        let content = fs::read_to_string(trig_path).unwrap();
        let diagram = Diagram::parse(&content).expect("Should parse trig_calc.dfpp");

        let mut runner = Runner::new();
        runner.start(&diagram);

        let mut steps = 0;
        while runner.state == ExecutionState::Running && steps < 500 {
            runner.step(&diagram);
            steps += 1;
        }

        assert_eq!(runner.state, ExecutionState::Finished);
        assert!(!runner.output_history.is_empty(), "Should capture numeric outputs");
        assert!(runner.variable_history.contains_key("cosx"), "Should record history for cosx");
        assert!(runner.variable_history.contains_key("siny"), "Should record history for siny");

        let cosx_val = runner.variables.get("cosx").map(|v| v.to_number()).unwrap_or(0.0);
        let siny_val = runner.variables.get("siny").map(|v| v.to_number()).unwrap_or(0.0);
        assert!((cosx_val >= -1.0 && cosx_val <= 1.0), "cosx should be in range [-1, 1]");
        assert!((siny_val >= -1.0 && siny_val <= 1.0), "siny should be in range [-1, 1]");
    }

    #[test]
    fn test_sin_cos_wave_execution() {
        let wave_path = Path::new("examples/sin_cos_wave.dfpp");
        assert!(wave_path.exists());

        let content = fs::read_to_string(wave_path).unwrap();
        let diagram = Diagram::parse(&content).expect("Should parse sin_cos_wave.dfpp");

        let mut runner = Runner::new();
        runner.start(&diagram);

        let mut steps = 0;
        while runner.state == ExecutionState::Running && steps < 1000 {
            runner.step(&diagram);
            steps += 1;
        }

        assert_eq!(runner.state, ExecutionState::Finished);
        assert!(!runner.output_history.is_empty(), "Outputs should be populated");
        assert!(runner.variable_history.contains_key("sinx"), "Should record sinx history");
        assert!(runner.variable_history.contains_key("cosx"), "Should record cosx history");
    }
}
