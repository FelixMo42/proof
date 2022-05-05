pub mod parser;
pub mod value;

use std::collections::HashMap;

use parser::AST;
use value::Value;

struct Scope(HashMap<String, Value>);

impl Scope {
    fn set(&mut self, name: &str, value: Value) {
        self.0.insert(name.to_string(), value);
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

fn ast_match(pattern: &AST, value: &Value, scope: &mut Scope) -> bool {
    match pattern {
        AST::Named(name) => {
            if let Some(current) = scope.get(&name) {
                return value == &current;
            } else {
                scope.set(name, value.clone());
                return true;
            }
        }
        AST::Value(current) => return current == value,
        AST::Braket(a, b) => {
            if let Value::Braket(va, vb) = value {
                return ast_match(a, va.as_ref(), scope) && ast_match(b, vb.as_ref(), scope);
            } else {
                return false;
            }
        }
        AST::Kind(name, avalue) => {
            if let Value::Kind(vname, vvalue) = value {
                if name == vname {
                    scope.set(avalue, Value::Number(*vvalue));
                    return true;
                }
            }
            return false;
        }
        AST::Negative(pattern) => {
            if let Value::Negative(value) = value {
                return ast_match(pattern, value, scope);
            }

            return false;
        }
        AST::Add(a, b) => {
            if let Value::Add(va, vb) = value {
                return (ast_match(a, va, scope) && ast_match(b, vb, scope))
                    || (ast_match(a, vb, scope) && ast_match(b, va, scope));
            }
            return false;
        }
        AST::Mul(a, b) => {
            if let Value::Mul(va, vb) = value {
                return (ast_match(a, va, scope) && ast_match(b, vb, scope))
                    || (ast_match(a, vb, scope) && ast_match(b, va, scope));
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

fn ast_number(expresion: &AST, scope: &Scope) -> usize {
    let value = ast_build(expresion, scope);

    if let Value::Number(num) = value {
        return num as usize;
    }

    panic!("Failed to get number for ast_number!");
}

fn ast_build(expresion: &AST, scope: &Scope) -> Value {
    match expresion {
        AST::Named(name) => scope.get(&name).expect(&format!("Could not find variable {name}!")),
        AST::Value(value) => value.clone(),
        AST::Kind(name, value) => Value::Kind(name.clone(), scope.get_number(&value)),
        AST::Negative(expresion) => Value::Negative(Box::new(ast_build(expresion, scope))),
        AST::Braket(a, b) => Value::Braket(
            Box::new(ast_build(a, scope)),
            Box::new(ast_build(b, scope))
        ),
        AST::Add(a, b) => Value::Add(
            Box::new(ast_build(a, scope)),
            Box::new(ast_build(b, scope))
        ),
        AST::Mul(a, b) => Value::Mul(
            Box::new(ast_build(a, scope)),
            Box::new(ast_build(b, scope))
        ),
        AST::C(a, b) => {
            let value = ARR[ast_number(a, scope) - 1][ast_number(b, scope) - 1];
            if value >= 0 {
                Value::Number(value)
            } else {
                Value::Negative(Box::new(Value::Number(-value)))
            }
        }
    }
}

fn ast_match_and_build(value: Value, patterns: &Vec<(AST, AST)>) -> Value {
    for (pattern, expresion) in patterns {
        let scope = &mut Scope(HashMap::new());

        if ast_match(&pattern, &value, scope) {
            return simplify(ast_build(expresion, scope), patterns);
        }

        if let Some(pattern) = pattern.flip() {
            if ast_match(&pattern, &value, scope) {
                return simplify(ast_build(&expresion.negate(), scope), patterns);
            }
        }
    }

    return value;
}

fn simplify(value: Value, patterns: &Vec<(AST, AST)>) -> Value {
    match value {
        Value::Add(a, b) => {
            return ast_match_and_build(Value::Add(Box::new(simplify(*a, patterns)), Box::new(simplify(*b, patterns))), patterns)
        }
        Value::Negative(value) => {
            return ast_match_and_build(Value::Negative(Box::new(simplify(*value, patterns))), patterns)
        }
        Value::Mul(a, b) => {    
            return ast_match_and_build(Value::Mul(Box::new(simplify(*a, patterns)), Box::new(simplify(*b, patterns))), patterns)
        }
        Value::Braket(a, b) => {
            return ast_match_and_build(Value::Braket(Box::new(simplify(*a, patterns)), Box::new(simplify(*b, patterns))), patterns)
        }
        Value::Number(_) => return value,
        Value::Kind(_, _) => return value,
    }
}

fn main() {
    let patterns = &parser::load("./src/map");
    let mut value = Value::Kind("E".to_string(), 1);

    for i in 1..100 {
        println!(">> {}", i);
        value = Value::Braket(Box::new(value.clone()), Box::new(Value::Kind("E".to_string(), i % 3 + 1)));
        for f in 1..=3 {
            let value = Value::Braket(Box::new(value.clone()), Box::new(Value::Kind("F".to_string(), f))); 
            if simplify(value, patterns) == Value::zero() {
                println!("zero! {i} {f}");
                return;
            }
        }
    }

    println!("no zero found");
}