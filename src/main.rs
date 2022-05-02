pub mod parser;
pub mod value;

use std::collections::HashMap;

use parser::AST;
use value::Value;

struct Scope(HashMap<String, Value>);

impl Scope {
    fn set(&mut self, name: String, value: Value) {
        self.0.insert(name, value);
    }

    fn get(&self, name: &str) -> Option<Value> {
        return self.0.get(name).cloned();
    }

    fn get_number(&self, name: &str) -> i32 {
        let value = self.get(name).expect(&format!("Could not find variable {name}!"));
    
        if let Value::Number(num) = value {
            return num
        } else {
            panic!("Expected a number for {name}, got {value}!")
        }
    }
}

fn ast_match(pattern: AST, value: &Value, scope: &mut Scope) -> bool {
    match pattern {
        AST::Named(name) => {
            if let Some(current) = scope.get(&name) {
                return value == &current;
            } else {
                scope.set(name, value.clone());
                return true;
            }
        }
        AST::Value(current) => return &current == value,
        AST::Braket(a, b) => {
            if let Value::Braket(va, vb) = value {
                return ast_match(*a, va.as_ref(), scope) && ast_match(*b, vb.as_ref(), scope);
            } else {
                return false;
            }
        }
        AST::Kind(name, avalue) => {
            if let Value::Kind(vname, vvalue) = value {
                if &name == vname {
                    scope.set(avalue, Value::Number(*vvalue));
                    return true;
                }
            }
            return false;
        }
        AST::Negative(pattern) => {
            if let Value::Negative(value) = value {
                return ast_match(*pattern, value, scope);
            }

            return false;
        }
        AST::Add(a, b) => {
            if let Value::Add(va, vb) = value {
                return ast_match(*a, va, scope) && ast_match(*b, vb, scope);
            }
            return false;
        }
        AST::Mul(a, b) => {
            if let Value::Mul(va, vb) = value {
                return ast_match(*a, va, scope) && ast_match(*b, vb, scope);
            }
            return false;
        }
        AST::C(_, _) => {
            panic!("C statments not allowed on left side of equal!")
        }
    }
}

const ARR: [[i32; 3]; 3] = [
    [  2, -1, -1 ],
    [ -1,  2, -2 ],
    [ -1, -1,  2 ]
];

fn ast_number(expresion: AST, scope: &Scope) -> usize {
    let value = ast_build(expresion, scope);

    if let Value::Number(num) = value {
        return num as usize;
    }

    panic!("Failed to get number for ast_number!");
}

fn ast_build(expresion: AST, scope: &Scope) -> Value {
    match expresion {
        AST::Named(name) => scope.get(&name).expect(&format!("Could not find variable {name}!")),
        AST::Value(value) => value,
        AST::Kind(name, value) => Value::Kind(name, scope.get_number(&value)),
        AST::Negative(expresion) => Value::Negative(Box::new(ast_build(*expresion, scope))),
        AST::Braket(a, b) => Value::Braket(
            Box::new(ast_build(*a, scope)),
            Box::new(ast_build(*b, scope))
        ),
        AST::Add(a, b) => Value::Add(
            Box::new(ast_build(*a, scope)),
            Box::new(ast_build(*b, scope))
        ),
        AST::Mul(a, b) => Value::Mul(
            Box::new(ast_build(*a, scope)),
            Box::new(ast_build(*b, scope))
        ),
        AST::C(a, b) => Value::Number(ARR[ast_number(*a, scope)][ast_number(*b, scope)])
    }
}

fn ast_match_and_build(value: &Value) -> Option<Value> {
    let patterns: Vec<(AST, AST)> = parser::load("src/map");

    for (pattern, expresion) in patterns {
        let scope = &mut Scope(HashMap::new());

        if ast_match(pattern, value, scope) {
            return Some(ast_build(expresion, scope));
        }
    }

    return None;
}

fn simplify(value: &Value) -> Value {
    if let Some(value) = ast_match_and_build(value) {
        return value;
    }

    match value {
        Value::Add(a, b) => {
            if let Value::Number(a) = **a {
                if let Value::Number(b) = **b {
                    return Value::Number(a + b);
                }
            }

            return Value::Add(Box::new(simplify(a)), Box::new(simplify(b)))
        }
        Value::Negative(value) => {
            if let Value::Negative(value) = *value.clone() {
                return simplify(&value);
            }

            return Value::Negative(Box::new(simplify(value)))
        }
        Value::Mul(a, b) => {
            if **a == Value::zero() || **b == Value::zero() {
                return Value::Number(0);
            }


            if let Value::Number(a) = **a {
                if let Value::Number(b) = **b {
                    return Value::Number(a * b);
                }
            }

            if **a == Value::one() {
                return simplify(b);
            }

            if **b == Value::one() {
                return simplify(a);
            }

            if let Value::Number(a) = **a {
                if a < 0 {
                    return Value::Negative(
                        Box::new(Value::Mul(Box::new(Value::Number(-a)), Box::new(simplify(b))))
                    )
                }
            }

            if let Value::Number(b) = **b {
                if b < 0 {
                    return Value::Negative(
                        Box::new(Value::Mul(Box::new(Value::Number(-b)), Box::new(simplify(a))))
                    )
                }
            }

            return Value::Mul(Box::new(simplify(a)), Box::new(simplify(b)))
        }
        Value::Braket(a, b) => {
            return Value::Braket(Box::new(simplify(a)), Box::new(simplify(b)))
        }
        Value::Number(_) | Value::Kind(_, _) => return value.clone(),
    }
}

fn source_build(src: &str) -> Value {
    return ast_build(parser::parse(src), &Scope(HashMap::new()));
}

fn main() {
    let mut value = source_build("[E(1), H(2)]");

    println!("  {}", value);

    loop {
        let new_value = simplify(&value);

        if new_value == value {
            break;
        }

        println!("= {}", new_value);
        value = new_value;
    }
}