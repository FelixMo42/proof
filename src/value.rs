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

    pub fn into_number(&self) -> Option<i32> {
        match self {
            Value::Number(num) => Some(*num),
            Value::Negative(num) => Some(-num.into_number()?),
            _ => None,
        }
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

#[inline]
pub fn brak(a: Value, b: Value) -> Value {
    return Value::Braket(Box::new(a), Box::new(b));
}

#[inline]
pub fn H(n: i32) -> Value {
    return Value::Kind("H".to_string(), n);
}

#[inline]
pub fn E(n: i32) -> Value {
    return Value::Kind("E".to_string(), n);
}

#[inline]
pub fn F(n: i32) -> Value {
    return Value::Kind("F".to_string(), n);
}

pub fn number(n: i32) -> Value {
    if n >= 0 {
        Value::Number(n)
    } else {
        Value::Negative(Box::new(Value::Number(-n)))
    }
}