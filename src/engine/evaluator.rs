use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl Value {
    pub fn to_string_val(&self) -> String {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::String(s) => s.clone(),
            Value::Bool(b) => format!("{}", b),
            Value::Nil => "".to_string(),
        }
    }

    pub fn to_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::String(s) => s.trim().parse().unwrap_or(0.0),
            Value::Bool(b) => if *b { 1.0 } else { 0.0 },
            Value::Nil => 0.0,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_val())
    }
}

pub struct Evaluator;

impl Evaluator {
    pub fn parse_value(expr: &str, vars: &HashMap<String, Value>) -> Value {
        let trimmed = expr.trim();
        if trimmed.is_empty() {
            return Value::Nil;
        }

        // String literal in double quotes or single quotes
        if ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
            && trimmed.len() >= 2
        {
            return Value::String(trimmed[1..trimmed.len() - 1].to_string());
        }

        // Numeric literal
        if let Ok(num) = trimmed.parse::<f64>() {
            return Value::Number(num);
        }

        // Boolean literal
        if trimmed.eq_ignore_ascii_case("true") {
            return Value::Bool(true);
        }
        if trimmed.eq_ignore_ascii_case("false") {
            return Value::Bool(false);
        }

        // Arithmetic expression (simple + - * /)
        if trimmed.contains('+') {
            let parts: Vec<&str> = trimmed.splitn(2, '+').collect();
            let left = Self::parse_value(parts[0], vars);
            let right = Self::parse_value(parts[1], vars);
            match (&left, &right) {
                (Value::String(s1), Value::String(s2)) => return Value::String(format!("{}{}", s1, s2)),
                (Value::String(s1), r) => return Value::String(format!("{}{}", s1, r.to_string_val())),
                (l, Value::String(s2)) => return Value::String(format!("{}{}", l.to_string_val(), s2)),
                _ => return Value::Number(left.to_number() + right.to_number()),
            }
        }

        if trimmed.contains('-') && !trimmed.starts_with('-') {
            let parts: Vec<&str> = trimmed.splitn(2, '-').collect();
            let left = Self::parse_value(parts[0], vars).to_number();
            let right = Self::parse_value(parts[1], vars).to_number();
            return Value::Number(left - right);
        }

        if trimmed.contains('*') {
            let parts: Vec<&str> = trimmed.splitn(2, '*').collect();
            let left = Self::parse_value(parts[0], vars).to_number();
            let right = Self::parse_value(parts[1], vars).to_number();
            return Value::Number(left * right);
        }

        if trimmed.contains('/') {
            let parts: Vec<&str> = trimmed.splitn(2, '/').collect();
            let left = Self::parse_value(parts[0], vars).to_number();
            let right = Self::parse_value(parts[1], vars).to_number();
            if right != 0.0 {
                return Value::Number(left / right);
            } else {
                return Value::Number(0.0);
            }
        }

        // Function call parsing: fn_name(arg) e.g. cos(x), sin(y), sqrt(x)
        if let Some(open_paren) = trimmed.find('(') {
            if trimmed.ends_with(')') {
                let func_name = trimmed[..open_paren].trim();
                let arg_inner = trimmed[open_paren + 1..trimmed.len() - 1].trim();

                if matches!(
                    func_name.to_lowercase().as_str(),
                    "cos" | "sin" | "tan" | "asin" | "acos" | "atan" | "abs" | "sqrt" | "fix" | "int" | "exp" | "log" | "ln" | "len"
                ) {
                    let arg_val = Self::parse_value(arg_inner, vars);
                    let mut temp_vars = vars.clone();
                    temp_vars.insert("__arg__".to_string(), arg_val);
                    return Self::eval_function(func_name, "__arg__", &temp_vars);
                }
            }
        }

        // 1. Exact variable lookup
        if let Some(val) = vars.get(trimmed) {
            return val.clone();
        }

        // 2. Case-insensitive variable lookup
        if let Some((_, val)) = vars.iter().find(|(k, _)| k.eq_ignore_ascii_case(trimmed)) {
            return val.clone();
        }

        // 3. Object property dot-notation resolution (e.g. var.prop -> var)
        if trimmed.contains('.') {
            let base_name = trimmed.split('.').next().unwrap_or(trimmed);
            if let Some((_, val)) = vars.iter().find(|(k, _)| k.eq_ignore_ascii_case(base_name)) {
                return val.clone();
            }
        }

        // 4. Fallback: treat unquoted text as literal string (no "" quotes required)
        Value::String(trimmed.to_string())
    }

    pub fn eval_equal(left_str: &str, right_str: &str, vars: &HashMap<String, Value>) -> bool {
        let left = Self::parse_value(left_str, vars);
        let right = Self::parse_value(right_str, vars);

        match (left, right) {
            (Value::Number(n1), Value::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (l, r) => l.to_string_val() == r.to_string_val(),
        }
    }

    pub fn eval_comparison(op: &str, left_str: &str, right_str: &str, vars: &HashMap<String, Value>) -> bool {
        let left = Self::parse_value(left_str, vars);
        let right = Self::parse_value(right_str, vars);

        match op.trim().to_uppercase().as_str() {
            "==" | "IF_EQUAL" | "EQUAL" => Self::eval_equal(left_str, right_str, vars),
            ">" | "IF_GREATER" | "GREATER" => left.to_number() > right.to_number(),
            ">=" | "IF_GREATER_EQUAL" => left.to_number() >= right.to_number(),
            "<" | "IF_LESS" | "LESS" => left.to_number() < right.to_number(),
            "<=" | "IF_LESS_EQUAL" => left.to_number() <= right.to_number(),
            _ => Self::eval_equal(left_str, right_str, vars),
        }
    }

    pub fn eval_function(func_name: &str, arg_str: &str, vars: &HashMap<String, Value>) -> Value {
        let arg = Self::parse_value(arg_str, vars);
        match func_name.trim().to_lowercase().as_str() {
            "cos" => Value::Number(arg.to_number().cos()),
            "sin" => Value::Number(arg.to_number().sin()),
            "tan" => Value::Number(arg.to_number().tan()),
            "asin" => Value::Number(arg.to_number().asin()),
            "acos" => Value::Number(arg.to_number().acos()),
            "atan" => Value::Number(arg.to_number().atan()),
            "exp" => Value::Number(arg.to_number().exp()),
            "log" | "ln" => Value::Number(arg.to_number().ln()),
            "fix" | "int" => Value::Number(arg.to_number().floor()),
            "abs" => Value::Number(arg.to_number().abs()),
            "sqrt" => Value::Number(arg.to_number().sqrt()),
            "len" | "length" => Value::Number(arg.to_string_val().len() as f64),
            "clear" => Value::Number(0.0),
            "f" | "fact" | "factorial" => {
                let n = arg.to_number().floor() as i64;
                if n <= 1 {
                    Value::Number(1.0)
                } else {
                    let mut fact = 1.0;
                    for i in 2..=n {
                        fact *= i as f64;
                    }
                    Value::Number(fact)
                }
            }
            _ => arg,
        }
    }
}
