use crate::engine::evaluator::{Evaluator, Value};
use crate::model::connector::BranchCondition;
use crate::model::node::{Node, NodeType};
use std::collections::HashMap;

/// Helper: Execute arithmetic operation (ADD, SUBTRACT, MULTIPLY, DIVIDE)
pub fn execute_arithmetic(
    node: &Node,
    vars: &mut HashMap<String, Value>,
) -> (String, Value, String) {
    let p1 = Evaluator::parse_value(&node.expr1, vars).to_number();
    let p2 = Evaluator::parse_value(&node.expr2, vars).to_number();
    let target = if !node.target_var.is_empty() {
        &node.target_var
    } else {
        &node.expr1
    };

    let (op_name, res_num) = match node.node_type {
        NodeType::Add => ("TOPLA", p1 + p2),
        NodeType::Subtract => ("ÇIKAR", p1 - p2),
        NodeType::Multiply => ("ÇARP", p1 * p2),
        NodeType::Divide => (
            "BÖL",
            if p2 != 0.0 && p2.is_finite() && p1.is_finite() {
                let result = p1 / p2;
                if result.is_finite() { result } else { 0.0 }
            } else {
                0.0
            },
        ),
        _ => ("İŞLEM", 0.0),
    };

    // Validate result to prevent NaN/Infinity
    let res_num = if res_num.is_finite() { res_num } else { 0.0 };
    let res = Value::Number(res_num);
    if !target.is_empty() {
        vars.insert(target.clone(), res.clone());
    }

    let log = format!(
        "[{}] `{}` ({}) {} ({}) -> {}",
        op_name,
        target,
        p1,
        match node.node_type {
            NodeType::Add => "+",
            NodeType::Subtract => "-",
            NodeType::Multiply => "*",
            NodeType::Divide => "/",
            _ => "?",
        },
        p2,
        res
    );

    (target.clone(), res, log)
}

/// Helper: Evaluate condition nodes (IF_EQUAL, IF_GREATER, etc.)
pub fn execute_condition(
    node: &Node,
    vars: &HashMap<String, Value>,
) -> (BranchCondition, String) {
    let val1 = Evaluator::parse_value(&node.expr1, vars);
    let val2 = Evaluator::parse_value(&node.expr2, vars);
    let is_true = Evaluator::eval_comparison(
        node.node_type.code_name(),
        &node.expr1,
        &node.expr2,
        vars,
    );

    let branch = if is_true {
        BranchCondition::Yes
    } else {
        BranchCondition::No
    };

    let log = format!(
        "[KOŞUL] {} `{}` [{}] vs `{}` [{}] -> {}",
        node.node_type.code_name(),
        node.expr1,
        val1,
        node.expr2,
        val2,
        if is_true { "EVET" } else { "HAYIR" }
    );

    (branch, log)
}

/// Helper: Execute variable definition node
pub fn execute_definition(
    node: &Node,
    vars: &mut HashMap<String, Value>,
) -> String {
    let e1 = node.expr1.trim();
    let e2 = node.expr2.trim();

    // Multi-statement comma-separated assignments: x = 0.5, y = 1.0, cx = cos(x), sy = sin(y)
    if e1.contains('=') {
        let mut logs = Vec::new();
        for statement in e1.split(',') {
            let stmt = statement.trim();
            if stmt.contains('=') {
                let parts: Vec<&str> = stmt.splitn(2, '=').collect();
                let var_name = parts[0].trim();
                let val_expr = parts[1].trim();
                let val = Evaluator::parse_value(val_expr, vars);
                if !var_name.is_empty() {
                    vars.insert(var_name.to_string(), val.clone());
                    vars.insert(format!("{}.Value", var_name), val.clone());
                }
                logs.push(format!("`{}` = {}", var_name, val));
            }
        }
        return format!("[TANIM] {}", logs.join(", "));
    }

    let val2 = Evaluator::parse_value(e2, vars);
    let val1 = Evaluator::parse_value(e1, vars);

    let val = if val2 != Value::Nil {
        val2
    } else if val1 != Value::Nil {
        val1
    } else {
        Value::Nil
    };

    let mut set_var = |var_name: &str, v: &Value| {
        if !var_name.is_empty() {
            vars.insert(var_name.to_string(), v.clone());
            if var_name.contains('.') {
                let base = var_name.split('.').next().unwrap_or(var_name);
                vars.insert(base.to_string(), v.clone());
            } else {
                vars.insert(format!("{}.Text", var_name), v.clone());
                vars.insert(format!("{}.Value", var_name), v.clone());
            }
        }
    };

    set_var(e1, &val);
    set_var(e2, &val);

    format!("[TANIM] `{}` = `{}` -> {}", e1, e2, val)
}

/// Helper: Execute action nodes (ACTION, JOIN, COMP)
pub fn execute_action(
    node: &Node,
    vars: &mut HashMap<String, Value>,
) -> String {
    let op_kind = node.label.trim().to_uppercase();
    match op_kind.as_str() {
        "JOIN" => {
            let p1 = Evaluator::parse_value(&node.expr1, vars).to_string_val();
            let p2 = Evaluator::parse_value(&node.expr2, vars).to_string_val();
            let joined = format!("{}{}", p1, p2);
            if !node.target_var.is_empty() {
                vars.insert(node.target_var.clone(), Value::String(joined.clone()));
            }
            format!(
                "[BİRLEŞTİR] `{}` + `{}` -> `{}` = {}",
                node.expr1, node.expr2, node.target_var, joined
            )
        }
        "COMP" => {
            let is_eq = Evaluator::eval_equal(&node.expr1, &node.expr2, vars);
            let comp_res = if is_eq { 0.0 } else { 1.0 };
            if !node.target_var.is_empty() {
                vars.insert(node.target_var.clone(), Value::Number(comp_res));
            }
            format!(
                "[KARŞILAŞTIR] `{}` vs `{}` -> `{}` = {}",
                node.expr1, node.expr2, node.target_var, comp_res
            )
        }
        _ => {
            let val = Evaluator::parse_value(&node.expr2, vars);
            let target = if !node.target_var.is_empty() {
                &node.target_var
            } else {
                &node.expr1
            };
            if !target.is_empty() {
                vars.insert(target.to_string(), val.clone());
            }
            format!("[ATAMA] `{}` = {}", target, val)
        }
    }
}
