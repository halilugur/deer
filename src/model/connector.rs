use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchCondition {
    Default,
    Yes,
    No,
}

impl BranchCondition {

    pub fn tag(&self) -> &'static str {
        match self {
            BranchCondition::Default => "",
            BranchCondition::Yes => "YES",
            BranchCondition::No => "NO",
        }
    }

    pub fn from_fpp_str(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "YES" | "EVET" => BranchCondition::Yes,
            "NO" | "HAYIR" => BranchCondition::No,
            _ => BranchCondition::Default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connector {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub condition: BranchCondition,
}

impl Connector {
    pub fn new(id: impl Into<String>, from_id: impl Into<String>, to_id: impl Into<String>, condition: BranchCondition) -> Self {
        Self {
            id: id.into(),
            from_id: from_id.into(),
            to_id: to_id.into(),
            condition,
        }
    }
}
