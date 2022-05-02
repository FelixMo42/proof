use std::fmt::Display;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    Number(i32),
    Braket(Box<Value>, Box<Value>),
    Kind(String, i32),
    Negative(Box<Value>),
    Add(Box<Value>, Box<Value>),
    Mul(Box<Value>, Box<Value>),
}

impl Value {
    pub fn zero() -> Value {
        return Value::Number(0)
    }

    pub fn one() -> Value {
        return Value::Number(1)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(number) => write!(f, "{number}"),
            Value::Braket(a, b) => write!(f, "[{a}, {b}]"),
            Value::Kind(name, value) => write!(f, "{name}({value})"),
            Value::Negative(value) => write!(f, "-{value}"),
            Value::Add(a, b) => write!(f, "{a} + {b}"),
            Value::Mul(a, b) => write!(f, "{a} * {b}"),
        }
    }
}