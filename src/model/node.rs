use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Start,
    Action,
    Definition,
    Add,
    Subtract,
    Multiply,
    Divide,
    Input,
    Output,
    IfEqual,
    IfGreater,
    IfGreaterEqual,
    IfLess,
    IfLessEqual,
    Intersection,
    Function,
    Stop,
}

impl NodeType {
    pub fn display_name(&self) -> &'static str {
        match self {
            NodeType::Start => "START (başla)",
            NodeType::Action => "ACTION (işlem)",
            NodeType::Definition => "DEFINITION (tanım)",
            NodeType::Add => "ADD (+ toplama)",
            NodeType::Subtract => "SUBTRACT (- çıkarma)",
            NodeType::Multiply => "MULTIPLY (* çarpma)",
            NodeType::Divide => "DIVIDE (/ bölme)",
            NodeType::Input => "INPUT (giriş)",
            NodeType::Output => "OUTPUT (çıkış)",
            NodeType::IfEqual => "IF_EQUAL (eşittir ==)",
            NodeType::IfGreater => "IF_GREATER (büyüktür >)",
            NodeType::IfGreaterEqual => "IF_GREATER_EQUAL (büyük-eşit >=)",
            NodeType::IfLess => "IF_LESS (küçüktür <)",
            NodeType::IfLessEqual => "IF_LESS_EQUAL (küçük-eşit <=)",
            NodeType::Intersection => "JOIN (kesişim)",
            NodeType::Function => "FUNCTION (fonksiyon)",
            NodeType::Stop => "STOP (dur)",
        }
    }

    pub fn is_condition(&self) -> bool {
        matches!(
            self,
            NodeType::IfEqual
                | NodeType::IfGreater
                | NodeType::IfGreaterEqual
                | NodeType::IfLess
                | NodeType::IfLessEqual
        )
    }

    pub fn code_name(&self) -> &'static str {
        match self {
            NodeType::Start => "START",
            NodeType::Action => "ACTION",
            NodeType::Definition => "DEFINITION",
            NodeType::Add => "ADD",
            NodeType::Subtract => "SUBTRACT",
            NodeType::Multiply => "MULTIPLY",
            NodeType::Divide => "DIVIDE",
            NodeType::Input => "INPUT",
            NodeType::Output => "OUTPUT",
            NodeType::IfEqual => "IF_EQUAL",
            NodeType::IfGreater => "IF_GREATER",
            NodeType::IfGreaterEqual => "IF_GREATER_EQUAL",
            NodeType::IfLess => "IF_LESS",
            NodeType::IfLessEqual => "IF_LESS_EQUAL",
            NodeType::Intersection => "INTERSECTION",
            NodeType::Function => "FUNCTION",
            NodeType::Stop => "STOP",
        }
    }

    pub fn from_fpp_kind_or_type(kind: &str, type_num: u32) -> Self {
        let trimmed = kind.trim().to_uppercase();
        match trimmed.as_str() {
            "START" => NodeType::Start,
            "STOP" | "END" => NodeType::Stop,
            "INPUT" | "IN" => NodeType::Input,
            "OUTPUT" | "OUT" => NodeType::Output,
            "IF_EQUAL" | "IF" => NodeType::IfEqual,
            "IF_GREATER" => NodeType::IfGreater,
            "IF_GREATER_EQUAL" => NodeType::IfGreaterEqual,
            "IF_LESS" => NodeType::IfLess,
            "IF_LESS_EQUAL" => NodeType::IfLessEqual,
            "DEFINITION" => NodeType::Definition,
            "ADD" => NodeType::Add,
            "SUBTRACT" => NodeType::Subtract,
            "MULTIPLY" => NodeType::Multiply,
            "DIVIDE" => NodeType::Divide,
            "INTERSECTION" | "JOIN_NODE" => NodeType::Intersection,
            "FUNCTION" | "FUNC" => NodeType::Function,
            "COMP" | "JOIN" | "ACTION" | "ACT" => NodeType::Action,
            _ => match type_num {
                2 => {
                    if trimmed.contains("STOP") || trimmed.contains("DUR") {
                        NodeType::Stop
                    } else {
                        NodeType::Start
                    }
                }
                3 => NodeType::Intersection,
                91 => NodeType::Output,
                92 => NodeType::IfEqual,
                93 => NodeType::Function,
                0 => NodeType::Action,
                _ => NodeType::Action,
            },
        }
    }

    pub fn default_size(&self) -> (f32, f32) {
        match self {
            NodeType::Start | NodeType::Stop => (90.0, 36.0),
            NodeType::Action
            | NodeType::Definition
            | NodeType::Add
            | NodeType::Subtract
            | NodeType::Multiply
            | NodeType::Divide => (130.0, 42.0),
            NodeType::Input | NodeType::Output => (140.0, 42.0),
            NodeType::IfEqual
            | NodeType::IfGreater
            | NodeType::IfGreaterEqual
            | NodeType::IfLess
            | NodeType::IfLessEqual => (110.0, 80.0),
            NodeType::Intersection => (14.0, 14.0),
            NodeType::Function => (130.0, 42.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default = "default_fill_color")]
    pub fill_color: [u8; 3],   // RGB
    #[serde(default = "default_border_color")]
    pub border_color: [u8; 3], // RGB
    #[serde(default)]
    pub label: String,         // Primary label or raw operation kind
    #[serde(default)]
    pub expr1: String,         // First parameter / variable / left operand / prompt
    #[serde(default)]
    pub expr2: String,         // Second parameter / right operand / input var
    #[serde(default)]
    pub target_var: String,    // Result variable name (for JOIN, COMP, ADD, etc.)
}

fn default_fill_color() -> [u8; 3] {
    [255, 255, 255]
}

fn default_border_color() -> [u8; 3] {
    [40, 40, 40]
}

impl Node {
    pub fn new(id: impl Into<String>, node_type: NodeType, x: f32, y: f32) -> Self {
        let (w, h) = node_type.default_size();
        let (label, expr1, expr2, target_var) = match node_type {
            NodeType::Start => ("START".to_string(), "".to_string(), "".to_string(), "".to_string()),
            NodeType::Stop => ("STOP".to_string(), "".to_string(), "".to_string(), "".to_string()),
            NodeType::Input => ("INPUT".to_string(), "Enter value".to_string(), "x".to_string(), "".to_string()),
            NodeType::Output => ("OUTPUT".to_string(), "Value is: ".to_string(), "x".to_string(), "".to_string()),
            NodeType::IfEqual => ("IF_EQUAL".to_string(), "x".to_string(), "0".to_string(), "".to_string()),
            NodeType::IfGreater => ("IF_GREATER".to_string(), "x".to_string(), "0".to_string(), "".to_string()),
            NodeType::IfGreaterEqual => ("IF_GREATER_EQUAL".to_string(), "x".to_string(), "0".to_string(), "".to_string()),
            NodeType::IfLess => ("IF_LESS".to_string(), "x".to_string(), "0".to_string(), "".to_string()),
            NodeType::IfLessEqual => ("IF_LESS_EQUAL".to_string(), "x".to_string(), "0".to_string(), "".to_string()),
            NodeType::Definition => ("DEFINITION".to_string(), "x".to_string(), "0".to_string(), "".to_string()),
            NodeType::Add => ("ADD".to_string(), "x".to_string(), "1".to_string(), "x".to_string()),
            NodeType::Subtract => ("SUBTRACT".to_string(), "x".to_string(), "1".to_string(), "x".to_string()),
            NodeType::Multiply => ("MULTIPLY".to_string(), "x".to_string(), "2".to_string(), "x".to_string()),
            NodeType::Divide => ("DIVIDE".to_string(), "x".to_string(), "2".to_string(), "x".to_string()),
            NodeType::Action => ("ACTION".to_string(), "x".to_string(), "x + 1".to_string(), "x".to_string()),
            NodeType::Intersection => ("INTERSECTION".to_string(), "".to_string(), "".to_string(), "".to_string()),
            NodeType::Function => ("FUNCTION".to_string(), "Fix".to_string(), "x".to_string(), "x".to_string()),
        };

        Self {
            id: id.into(),
            node_type,
            x,
            y,
            width: w,
            height: h,
            fill_color: [255, 255, 255],
            border_color: [40, 40, 40],
            label,
            expr1,
            expr2,
            target_var,
        }
    }
}
