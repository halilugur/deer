use super::evaluator::{Evaluator, Value};
use crate::model::connector::BranchCondition;
use crate::model::diagram::Diagram;
use crate::model::node::{Node, NodeType};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const MAX_LOGS: usize = 1000;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionState {
    Idle,
    Running,
    Paused,
    WaitingForInput {
        prompt: String,
        target_var: String,
        next_node_id: String,
    },
    Finished,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub return_node_id: String,
    pub parent_variables: HashMap<String, Value>,
    pub return_target_var: String,
    pub parent_diagram: Diagram,
    pub parent_node_map: HashMap<String, Node>,
}

pub struct Runner {
    pub state: ExecutionState,
    pub current_node_id: Option<String>,
    pub active_diagram: Diagram,
    pub node_map: HashMap<String, Node>,
    pub diagram_stack: Vec<StackFrame>,
    pub variables: HashMap<String, Value>,
    pub logs: Vec<String>,
    pub output_history: Vec<(usize, f64)>,
    pub variable_history: HashMap<String, Vec<(usize, f64)>>,
    pub delay_ms: u64,
    pub step_count: usize,
    pub step_mode: bool,
    pub input_text: String,
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

impl Runner {
    pub fn new() -> Self {
        Self {
            state: ExecutionState::Idle,
            current_node_id: None,
            active_diagram: Diagram::default(),
            node_map: HashMap::new(),
            diagram_stack: Vec::new(),
            variables: HashMap::new(),
            logs: Vec::new(),
            output_history: Vec::new(),
            variable_history: HashMap::new(),
            delay_ms: 10, // Fast default execution speed (10ms)
            step_count: 0,
            step_mode: false,
            input_text: String::new(),
        }
    }

    pub fn push_log(&mut self, msg: String) {
        if self.logs.len() >= MAX_LOGS {
            self.logs.remove(0);
        }
        self.logs.push(msg);
    }

    pub fn reset(&mut self) {
        self.state = ExecutionState::Idle;
        self.current_node_id = None;
        self.active_diagram = Diagram::default();
        self.node_map.clear();
        self.diagram_stack.clear();
        self.variables.clear();
        self.logs.clear();
        self.output_history.clear();
        self.variable_history.clear();
        self.step_count = 0;
        self.step_mode = false;
        self.input_text.clear();
    }

    pub fn rebuild_node_map(&mut self) {
        self.node_map.clear();
        for node in &self.active_diagram.nodes {
            self.node_map.insert(node.id.clone(), node.clone());
        }
    }

    pub fn start(&mut self, diagram: &Diagram) {
        self.reset();
        self.active_diagram = diagram.clone();
        self.rebuild_node_map();
        self.step_mode = false;

        if let Some(start_node) = self.active_diagram.nodes.iter().find(|n| n.node_type == NodeType::Start) {
            self.current_node_id = Some(start_node.id.clone());
            self.state = ExecutionState::Running;

            // Initialize declared variables in Start node if any
            if !start_node.expr1.is_empty() {
                for var_name in start_node.expr1.split(',') {
                    let clean = var_name.trim();
                    if !clean.is_empty() {
                        self.variables.insert(clean.to_string(), Value::Number(0.0));
                    }
                }
            }

            self.push_log(format!("[Başlatıldı] Düğüm {} üzerinde çalışmaya başlandı", start_node.id));
        } else {
            self.state = ExecutionState::Error("No START node found in diagram!".to_string());
            self.push_log("[Hata] BAŞLA düğümü bulunamadı".to_string());
        }
    }

    pub fn stop(&mut self) {
        self.state = ExecutionState::Idle;
        self.current_node_id = None;
        self.step_mode = false;
        self.push_log("[Durduruldu] Çalışma kullanıcı tarafından durduruldu.".to_string());
    }

    pub fn pause(&mut self) {
        if self.state == ExecutionState::Running {
            self.state = ExecutionState::Paused;
            self.push_log("[Duraklatıldı] Çalışma duraklatıldı.".to_string());
        }
    }

    pub fn resume(&mut self) {
        if self.state == ExecutionState::Paused {
            self.step_mode = false;
            self.state = ExecutionState::Running;
            self.push_log("[Devam Ediyor] Çalışma devam ettirildi.".to_string());
        }
    }

    pub fn parse_user_input(input_str: &str) -> Value {
        let trimmed = input_str.trim();
        if trimmed.is_empty() {
            return Value::String("".to_string());
        }
        if ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
            && trimmed.len() >= 2
        {
            return Value::String(trimmed[1..trimmed.len() - 1].to_string());
        }
        if let Ok(num) = trimmed.parse::<f64>() {
            return Value::Number(num);
        }
        if trimmed.eq_ignore_ascii_case("true") {
            return Value::Bool(true);
        }
        if trimmed.eq_ignore_ascii_case("false") {
            return Value::Bool(false);
        }
        Value::String(trimmed.to_string())
    }

    pub fn submit_input(&mut self, input_str: &str, target_var: &str, next_node_id: &str) {
        let val = Self::parse_user_input(input_str);
        self.variables.insert(target_var.to_string(), val.clone());
        if target_var.contains('.') {
            let base = target_var.split('.').next().unwrap_or(target_var);
            self.variables.insert(base.to_string(), val.clone());
        } else {
            self.variables.insert(format!("{}.Text", target_var), val.clone());
            self.variables.insert(format!("{}.Value", target_var), val.clone());
        }
        self.push_log(format!("[Giriş] `{}` = {}", target_var, val));
        self.current_node_id = Some(next_node_id.to_string());
        if self.step_mode {
            self.state = ExecutionState::Paused;
        } else {
            self.state = ExecutionState::Running;
        }
        self.input_text.clear();
    }

    pub fn step_single(&mut self, root_diagram: &Diagram) {
        self.step_mode = true;

        if self.state == ExecutionState::Idle || self.state == ExecutionState::Finished {
            self.start(root_diagram);
            self.step_mode = true;
            if self.state == ExecutionState::Running {
                self.step(root_diagram);
                if self.state == ExecutionState::Running {
                    self.state = ExecutionState::Paused;
                }
            }
            return;
        }

        if let ExecutionState::WaitingForInput {
            prompt: _,
            target_var,
            next_node_id,
        } = self.state.clone()
        {
            let input_val = self.input_text.clone();
            self.submit_input(&input_val, &target_var, &next_node_id);
            return;
        }

        let was_paused = self.state == ExecutionState::Paused;
        if was_paused {
            self.state = ExecutionState::Running;
        }

        self.step(root_diagram);

        if self.state == ExecutionState::Running {
            self.state = ExecutionState::Paused;
        }
    }

    pub fn step(&mut self, root_diagram: &Diagram) {
        if self.state == ExecutionState::Idle && self.current_node_id.is_none() {
            self.start(root_diagram);
            return;
        }

        if self.state == ExecutionState::Finished
            || matches!(self.state, ExecutionState::Error(_))
            || matches!(self.state, ExecutionState::WaitingForInput { .. })
        {
            return;
        }

        let curr_id = match &self.current_node_id {
            Some(id) => id.clone(),
            None => return,
        };

        let node = match self.node_map.get(&curr_id) {
            Some(n) => n.clone(),
            None => {
                self.state = ExecutionState::Error(format!("Node {} not found", curr_id));
                return;
            }
        };

        self.step_count += 1;
        let mut next_node_id: Option<String> = None;
        let mut chosen_condition = BranchCondition::Default;

        match node.node_type {
            NodeType::Start => {
                // Advance to outgoing node
            }
            NodeType::Stop => {
                if let Some(frame) = self.diagram_stack.pop() {
                    // Returning from a sub-diagram function call
                    let return_val = self
                        .variables
                        .get("RETURN")
                        .or_else(|| self.variables.get("b"))
                        .or_else(|| self.variables.get("s"))
                        .cloned()
                        .unwrap_or(Value::Nil);

                    self.active_diagram = frame.parent_diagram;
                    self.node_map = frame.parent_node_map;
                    self.variables = frame.parent_variables;
                    if !frame.return_target_var.is_empty() {
                        self.variables.insert(frame.return_target_var.clone(), return_val.clone());
                    }

                    self.push_log(format!(
                        "[Geri Dönüldü] Fonksiyon `{}` -> `{}` = {}",
                        frame.function_name, frame.return_target_var, return_val
                    ));
                    self.current_node_id = Some(frame.return_node_id);
                    return;
                } else {
                    self.state = ExecutionState::Finished;
                    self.push_log("[Tamamlandı] Diyagram çalışması başarıyla tamamlandı.".to_string());
                    self.current_node_id = None;
                    return;
                }
            }
            NodeType::Input => {
                let prompt = if node.expr1.is_empty() {
                    format!("Enter value for {}", node.expr2)
                } else {
                    node.expr1.clone()
                };
                let target_var = if node.expr2.is_empty() {
                    "input_var".to_string()
                } else {
                    node.expr2.clone()
                };

                // Find next node
                if let Some(conn) = self.active_diagram.connectors.iter().find(|c| c.from_id == curr_id) {
                    self.state = ExecutionState::WaitingForInput {
                        prompt,
                        target_var,
                        next_node_id: conn.to_id.clone(),
                    };
                    return;
                } else {
                    self.state = ExecutionState::Finished;
                    return;
                }
            }
            NodeType::Output => {
                let text_prompt = &node.expr1;
                let var_name = &node.expr2;

                let (output_msg, val_to_check) = if !text_prompt.is_empty() && !var_name.is_empty() {
                    let val = Evaluator::parse_value(var_name, &self.variables);
                    (format!("{} {}", text_prompt, val), val.clone())
                } else if !text_prompt.is_empty() {
                    let val = Evaluator::parse_value(text_prompt, &self.variables);
                    (val.to_string_val(), val.clone())
                } else if !var_name.is_empty() {
                    let val = Evaluator::parse_value(var_name, &self.variables);
                    (val.to_string_val(), val.clone())
                } else {
                    ("".to_string(), Value::Nil)
                };

                if let Value::Number(num) = val_to_check {
                    self.output_history.push((self.step_count, num));
                } else if let Value::String(ref s) = val_to_check {
                    if let Ok(num) = s.trim().parse::<f64>() {
                        self.output_history.push((self.step_count, num));
                    }
                }

                self.push_log(format!("[Çıktı] {}", output_msg));
            }
            NodeType::Definition => {
                let log = super::node_exec::execute_definition(&node, &mut self.variables);
                self.push_log(log);
            }
            NodeType::Add | NodeType::Subtract | NodeType::Multiply | NodeType::Divide => {
                let (_target, _val, log) = super::node_exec::execute_arithmetic(&node, &mut self.variables);
                self.push_log(log);
            }
            NodeType::IfEqual
            | NodeType::IfGreater
            | NodeType::IfGreaterEqual
            | NodeType::IfLess
            | NodeType::IfLessEqual => {
                let (branch, log) = super::node_exec::execute_condition(&node, &self.variables);
                chosen_condition = branch;
                self.push_log(log);
            }
            NodeType::Action => {
                let log = super::node_exec::execute_action(&node, &mut self.variables);
                self.push_log(log);
            }
            NodeType::Function => {
                let func_name = &node.expr1;
                let arg_name = &node.expr2;
                let target_var = &node.target_var;

                // Check built-in function
                if matches!(func_name.to_lowercase().as_str(), "fix" | "int" | "abs" | "sqrt" | "clear" | "f" | "fact" | "factorial") {
                    let res_val = Evaluator::eval_function(func_name, arg_name, &self.variables);
                    let target = if !target_var.is_empty() {
                        target_var
                    } else {
                        arg_name
                    };
                    if !target.is_empty() {
                        self.variables.insert(target.to_string(), res_val.clone());
                        self.push_log(format!("[FONKSİYON] Dahili {}({}) -> `{}` = {}", func_name, arg_name, target, res_val));
                    }
                } else {
                    // Custom sub-diagram function lookup
                    let candidate_paths = [
                        PathBuf::from(format!("{}.fpp", func_name)),
                        PathBuf::from(format!("example/{}.fpp", func_name)),
                    ];

                    let mut loaded_sub_diagram: Option<Diagram> = None;
                    for path in &candidate_paths {
                        if path.exists() {
                            if let Ok(content) = fs::read_to_string(path) {
                                if let Ok(sub) = Diagram::parse_fpp(&content) {
                                    loaded_sub_diagram = Some(sub);
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(sub_diag) = loaded_sub_diagram {
                        let arg_val = Evaluator::parse_value(arg_name, &self.variables);

                        // Find next node in parent to return to
                        let return_node_id = self.active_diagram
                            .connectors
                            .iter()
                            .find(|c| c.from_id == curr_id)
                            .map(|c| c.to_id.clone())
                            .unwrap_or_else(|| curr_id.clone());

                        let parent_vars = self.variables.clone();
                        let parent_diagram = self.active_diagram.clone();
                        let parent_node_map = self.node_map.clone();

                        // Push stack frame
                        self.diagram_stack.push(StackFrame {
                            function_name: func_name.clone(),
                            return_node_id,
                            parent_variables: parent_vars,
                            return_target_var: target_var.clone(),
                            parent_diagram,
                            parent_node_map,
                        });

                        // Switch active diagram to loaded sub-diagram
                        self.active_diagram = sub_diag;
                        self.rebuild_node_map();

                        // Init sub-diagram environment
                        self.variables.clear();
                        self.variables.insert("PARAM".to_string(), arg_val.clone());
                        self.variables.insert(arg_name.clone(), arg_val.clone());

                        if let Some(start_node) = self.active_diagram.nodes.iter().find(|n| n.node_type == NodeType::Start) {
                            self.current_node_id = Some(start_node.id.clone());
                            self.push_log(format!("[ÇAĞRI] Alt-diyagram `{}` arg: {}", func_name, arg_val));
                            return;
                        }
                    } else {
                        self.push_log(format!("[UYARI] Fonksiyon `{}` bulunamadı.", func_name));
                    }
                }
            }
            NodeType::Intersection => {
                // Pass-through node
            }
        }

        // Find next outgoing connection in active diagram
        let outgoing: Vec<&crate::model::connector::Connector> = self.active_diagram
            .connectors
            .iter()
            .filter(|c| c.from_id == curr_id)
            .collect();

        if node.node_type.is_condition() {
            // Find branch matching YES or NO
            if let Some(conn) = outgoing.iter().find(|c| c.condition == chosen_condition) {
                next_node_id = Some(conn.to_id.clone());
            } else if let Some(conn) = outgoing.first() {
                next_node_id = Some(conn.to_id.clone());
            }
        } else if let Some(conn) = outgoing.first() {
            next_node_id = Some(conn.to_id.clone());
        }

        // Record numeric variable states into history for charting
        for (var_name, val) in &self.variables {
            if let Value::Number(num) = val {
                self.variable_history
                    .entry(var_name.clone())
                    .or_default()
                    .push((self.step_count, *num));
            }
        }

        if let Some(nid) = next_node_id {
            self.current_node_id = Some(nid);
        } else {
            self.state = ExecutionState::Finished;
            self.current_node_id = None;
            self.push_log("[Tamamlandı] Dal sonuna ulaşıldı.".to_string());
        }
    }
}
