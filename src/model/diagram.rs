use super::connector::{BranchCondition, Connector};
use super::node::{Node, NodeType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Diagram {
    #[serde(default)]
    pub title: String,
    pub nodes: Vec<Node>,
    pub connectors: Vec<Connector>,
    #[serde(default)]
    pub next_node_id: usize,
    #[serde(default)]
    pub next_line_id: usize,
}

fn is_id_header(line: &str) -> bool {
    let s = line.trim();
    if let Some(rest) = s.strip_prefix("id") {
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

impl Diagram {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            nodes: Vec::new(),
            connectors: Vec::new(),
            next_node_id: 1,
            next_line_id: 1,
        }
    }

    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn add_node(&mut self, node_type: NodeType, x: f32, y: f32) -> String {
        let id = format!("id{}", self.next_node_id);
        self.next_node_id += 1;
        let node = Node::new(&id, node_type, x, y);
        self.nodes.push(node);
        id
    }

    pub fn delete_node(&mut self, id: &str) {
        self.nodes.retain(|n| n.id != id);
        self.connectors.retain(|c| c.from_id != id && c.to_id != id);
    }

    pub fn add_connector(&mut self, from_id: &str, to_id: &str, condition: BranchCondition) -> Option<String> {
        if from_id == to_id {
            return None;
        }
        if self.connectors.iter().any(|c| c.from_id == from_id && c.to_id == to_id) {
            return None;
        }
        let line_id = format!("line{}", self.next_line_id);
        self.next_line_id += 1;
        self.connectors.push(Connector::new(&line_id, from_id, to_id, condition));
        Some(line_id)
    }

    pub fn delete_connector(&mut self, id: &str) {
        self.connectors.retain(|c| c.id != id);
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        let trimmed = content.trim_start();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            Self::parse_json(content)
        } else {
            Self::parse_fpp(content)
        }
    }

    pub fn parse_json(content: &str) -> Result<Self, String> {
        serde_json::from_str::<Diagram>(content)
            .map_err(|e| format!("JSON parsing error: {}", e))
    }

    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("JSON serialization error: {}", e))
    }

    pub fn parse_fpp(content: &str) -> Result<Self, String> {
        let mut diagram = Diagram::new("FPP Diagram");
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Ok(diagram);
        }

        let mut idx = 0;
        // Skip leading empty lines
        while idx < lines.len() && lines[idx].trim().is_empty() {
            idx += 1;
        }

        if idx >= lines.len() {
            return Ok(diagram);
        }

        let mut max_id_num = 0;

        let num_shapes: usize = lines
            .get(idx)
            .and_then(|l| l.split_whitespace().next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        idx += 1;

        while idx < lines.len() && (lines[idx].trim().is_empty() || lines[idx].contains("LINES")) {
            idx += 1;
        }

        for _ in 0..num_shapes {
            while idx < lines.len() && !is_id_header(lines[idx]) && !lines[idx].contains("---- LINES") {
                idx += 1;
            }

            if idx >= lines.len() || lines[idx].contains("---- LINES") {
                break;
            }

            let id = lines[idx].trim().to_string();
            if let Some(num) = id.strip_prefix("id").and_then(|n| n.parse::<usize>().ok()) {
                if num > max_id_num {
                    max_id_num = num;
                }
            }
            idx += 1;

            let mut type_num = 0;
            let mut left = 100.0f32;
            let mut top = 100.0f32;
            let mut width = 100.0f32;
            let mut height = 40.0f32;

            if idx < lines.len() {
                type_num = lines[idx].split_whitespace().next().unwrap_or("0").parse().unwrap_or(0);
                idx += 1;
            }
            if idx < lines.len() {
                left = lines[idx].split_whitespace().next().unwrap_or("100").parse().unwrap_or(100.0);
                idx += 1;
            }
            if idx < lines.len() {
                top = lines[idx].split_whitespace().next().unwrap_or("100").parse().unwrap_or(100.0);
                idx += 1;
            }
            if idx < lines.len() {
                width = lines[idx].split_whitespace().next().unwrap_or("100").parse().unwrap_or(100.0);
                idx += 1;
            }
            if idx < lines.len() {
                height = lines[idx].split_whitespace().next().unwrap_or("40").parse().unwrap_or(40.0);
                idx += 1;
            }

            // Skip backcolor, bordercolor, bordercolor2, reserved 1, reserved 2 (5 lines)
            for _ in 0..5 {
                if idx < lines.len() {
                    idx += 1;
                }
            }

            let kind = if idx < lines.len() {
                let k = lines[idx].trim().to_string();
                idx += 1;
                k
            } else {
                "ACTION".to_string()
            };

            let node_type = NodeType::from_fpp_kind_or_type(&kind, type_num);

            let mut p1 = String::new();
            let mut p2 = String::new();
            let mut p3 = String::new();
            let mut param_index = 0;

            // Read parameters until next shape header or LINES block
            while idx < lines.len() {
                let line = lines[idx].trim();
                if is_id_header(line) || line.contains("---- LINES") {
                    break;
                }
                match param_index {
                    0 => p1 = line.to_string(),
                    1 => p2 = line.to_string(),
                    2 => p3 = line.to_string(),
                    _ => {}
                }
                param_index += 1;
                idx += 1;
            }

            let mut node = Node::new(&id, node_type, left, top);
            node.width = width;
            node.height = height;
            node.label = kind;

            match node_type {
                NodeType::Start => {
                    node.expr1 = p1;
                }
                NodeType::Input => {
                    node.expr1 = p1; // Prompt
                    node.expr2 = p2; // Variable name
                }
                NodeType::Output => {
                    node.expr1 = p1; // Prompt/Variable
                    node.expr2 = p2;
                }
                NodeType::IfEqual
                | NodeType::IfGreater
                | NodeType::IfGreaterEqual
                | NodeType::IfLess
                | NodeType::IfLessEqual => {
                    node.expr1 = p1; // left operand
                    node.expr2 = p2; // right operand
                }
                NodeType::Definition => {
                    node.expr1 = p1; // variable name
                    node.expr2 = p2; // value expression
                    node.target_var = p3;
                }
                NodeType::Add
                | NodeType::Subtract
                | NodeType::Multiply
                | NodeType::Divide
                | NodeType::Action => {
                    node.expr1 = p1;
                    node.expr2 = p2;
                    node.target_var = p3;
                }
                NodeType::Function => {
                    node.expr1 = p1; // Func name
                    node.expr2 = p2; // Arg
                    node.target_var = p3; // Target var
                }
                _ => {}
            }

            diagram.nodes.push(node);
        }

        diagram.next_node_id = max_id_num + 1;

        // Parse connector lines section
        while idx < lines.len() && !lines[idx].contains("---- LINES") {
            idx += 1;
        }

        if idx < lines.len() {
            idx += 1; // skip header line
            while idx < lines.len() {
                let line = lines[idx].trim();
                idx += 1;

                if line.is_empty() {
                    continue;
                }

                if line.contains(',') {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 2 {
                        let from_id = parts[0];
                        let to_id = parts[1];

                        // Skip reserved 1 line
                        if idx < lines.len() && (lines[idx].trim().is_empty() || lines[idx].trim().starts_with("reserved")) {
                            idx += 1;
                        }

                        // Check if next line is a condition tag (YES/NO)
                        let mut cond = BranchCondition::Default;
                        if idx < lines.len() {
                            let next_line = lines[idx].trim();
                            if next_line == "YES" || next_line == "NO" || next_line == "EVET" || next_line == "HAYIR" {
                                cond = BranchCondition::from_fpp_str(next_line);
                                idx += 1;
                            }
                        }

                        diagram.add_connector(from_id, to_id, cond);
                    }
                }
            }
        }

        Ok(diagram)
    }

    pub fn export_fpp(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("{}       \t <--SHAPES\n", self.nodes.len()));
        out.push_str(&format!("{}       \t <--LINES\n", self.connectors.len()));

        for node in &self.nodes {
            out.push_str(&format!("{}\n", node.id));
            let type_num = match node.node_type {
                NodeType::Start | NodeType::Stop => 2,
                NodeType::Intersection => 3,
                NodeType::Output => 91,
                NodeType::IfEqual
                | NodeType::IfGreater
                | NodeType::IfGreaterEqual
                | NodeType::IfLess
                | NodeType::IfLessEqual => 92,
                NodeType::Function => 93,
                _ => 0,
            };
            out.push_str(&format!("{}       \t <--TYPE\n", type_num));
            out.push_str(&format!("{}       \t <--LEFT\n", node.x as i32));
            out.push_str(&format!("{}       \t <--TOP\n", node.y as i32));
            out.push_str(&format!("{}       \t <--WIDTH\n", node.width as i32));
            out.push_str(&format!("{}       \t <--HEIGHT\n", node.height as i32));
            out.push_str("16777215       \t <--BACKCOLOR\n");
            out.push_str("0       \t <--BORDERCOLOR\n");
            out.push_str("0       \t <--BORDERCOLOR\n");
            out.push_str("-reserved 1-\n");
            out.push_str("-reserved 2-\n");
            out.push_str(&format!("{}\n", node.node_type.code_name()));

            if !node.expr1.is_empty() {
                out.push_str(&format!("{}\n", node.expr1));
            }
            if !node.expr2.is_empty() {
                out.push_str(&format!("{}\n", node.expr2));
            }
            if !node.target_var.is_empty() {
                out.push_str(&format!("{}\n", node.target_var));
            }
            out.push('\n');
        }

        out.push_str("\n  \n---- LINES ---- from,to ----\n");
        for conn in &self.connectors {
            out.push_str(&format!("{},{}\n", conn.from_id, conn.to_id));
            out.push_str("reserved 1\n");
            if conn.condition != BranchCondition::Default {
                out.push_str(&format!("{}\n", conn.condition.tag()));
            }
            out.push('\n');
        }

        out
    }
}
