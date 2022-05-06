pub mod parser;
pub mod value;
pub mod v2;

use std::collections::HashMap;

use parser::AST;
use value::*;

pub struct Scope(HashMap<String, Value>);

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
                    if let Some(avalue) = scope.get(avalue) {
                        return avalue == Value::Number(*vvalue); 
                    }

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
                return ast_match(a, va, scope) && ast_match(b, vb, scope);
            }
            return false;
        }
        AST::Mul(a, b) => {
            if let Value::Mul(va, vb) = value {
                return ast_match(a, va, scope) && ast_match(b, vb, scope);
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
            if let Some(a) = a.into_number() {
                if let Some(b) = b.into_number() {
                    return number(a * b);
                }
            }

            return ast_match_and_build(Value::Mul(Box::new(simplify(*a, patterns)), Box::new(simplify(*b, patterns))), patterns)
        }
        Value::Braket(a, b) => {
            return ast_match_and_build(Value::Braket(Box::new(simplify(*a, patterns)), Box::new(simplify(*b, patterns))), patterns)
        }
        Value::Number(_) => return value,
        Value::Kind(_, _) => return value,
    }
}

fn standerdize(value: Value) -> Value {
    match value {
        Value::Braket(a, b) => {
            match (a.as_ref(), b.as_ref()) {
                (Value::Kind(_, na), Value::Kind(_, nb)) => {
                    if na > nb {
                        Value::Braket(a, b)
                    } else {
                        Value::Negative(Box::new(Value::Braket(b, a)))
                    }
                }
                (k @ Value::Kind(_, _),  b @ Value::Braket(_, _)) => {
                    match standerdize(b.clone()) {
                        Value::Negative(b) => Value::Braket(Box::new(standerdize(*b)), Box::new(k.clone())),
                        b => Value::Negative(Box::new(Value::Braket(Box::new(standerdize(b.clone())), Box::new(k.clone())))),
                    }
                }
                (b @ Value::Braket(_, _), k @ Value::Kind(_, _)) => {
                    match standerdize(b.clone()) {
                        Value::Negative(b) => Value::Negative(Box::new(Value::Braket(Box::new(standerdize(*b)), Box::new(k.clone())))),
                        b => Value::Braket(Box::new(standerdize(b.clone())), Box::new(k.clone())),
                    }
                }
                _ => Value::Braket(a, b)
            }
        }
        _ => value
    }
}

fn is_lots_of_es_zero(mut value: Value) -> Value {
    // println!("s: {value}");
    let mut es: Vec<(i32, Value)> = vec! [];

    loop {
        match value {
            Value::Add(a, b) => {
                match *a {
                    Value::Negative(ref value) => {
                        match *value.clone() {
                            Value::Mul(n, e) => es.push((-n.into_number().expect("expected number"), *e.clone())),
                            v => es.push((-1, v)),
                        }
                    }
                    Value::Mul(n, e) => es.push((n.into_number().expect("expected number"), *e)),
                    value => es.push((1, value)),
                }
                value = *b;
            }
            Value::Mul(ref n, ref e) => {
                es.push((n.clone().into_number().expect("expected number"), *e.clone()));
                break;
            }
            Value::Negative(ref value) => {
                match *value.clone() {
                    Value::Mul(n, e) => es.push((-n.into_number().expect("expected number"), *e.clone())),
                    v => es.push((-1, v)),
                }
                break;
            }
            value => {
                es.push((1, value));
                break;
            }
        }
    }

    es = es.into_iter().map(|(n, e)| match standerdize(e) {
        Value::Negative(e) => (-n, *e),
        e => (n, e),
    }).collect();

    for i in (1..es.len()).rev() {
        for j in 0..i {
            if es[i].1 == es[j].1 {
                es[j].0 += es[i].0;
                es.remove(i);
                break;
            }
        }
    }

    // for (i, e) in &es {
    //     println!("-> {i} * {e}")
    // }

    return es.into_iter()
        .filter(|(n, _)| n != &0)
        .map(|(n, e)| if n == 1 {
            e
        } else if n == -1 {
            Value::Negative(Box::new(e))
        } else if n > 0 {
            Value::Mul(Box::new(Value::Number(n)), Box::new(e))
        } else {
            Value::Negative(Box::new(Value::Mul(Box::new(Value::Number(-n)), Box::new(e))))
        })
        .reduce(|p, v| Value::Add(Box::new(p), Box::new(v)))
        .unwrap_or(Value::zero());
}

pub fn str_build(str: &str) -> Value {
    return ast_build(&parser::parse(str), &Scope(HashMap::new()));
}

pub fn check_conter_example_from_string(str: &str) {
    let ast = str_build(str);
    let patterns = &parser::load("./src/map");
    let value = is_lots_of_es_zero(simplify(ast, patterns));
    println!("{value}");
}

pub fn make(src: &str, patterns: &Vec<(AST, AST)>, scope: &Scope) -> Value {
    is_lots_of_es_zero(simplify(ast_build(&parser::parse(src), scope), patterns))
}

pub fn n_f(n: i32, b: i32) -> Value {
    let patterns = &parser::load("./src/map");

    let mut scope = Scope(HashMap::new());
    scope.0.insert("b".to_string(), Value::Number(b));

    let mut nx   = make("[E(1), E(2)]", patterns, &scope);
    let mut nx_f = make("[[E(1), E(2)], F(b)]", patterns, &scope);
    let mut nx_h = make("[[E(1), E(2)], H(b)]", patterns, &scope);

    for n in 2..n {
        scope.0.insert("n".to_string(), Value::Number(n % 3 + 1));
        scope.0.insert("nx".to_string(), nx.clone());
        scope.0.insert("nx_f".to_string(), nx_f);
        scope.0.insert("nx_h".to_string(), nx_h);

        nx = brak(nx, e(n % 3 + 1)); 

        nx_f = if n & 3 + 1 == b {
            make("[nx_f, E(n)] - nx_h ", patterns, &scope)
        } else {
            make("[nx_f, E(n)]", patterns, &scope)
        };

        nx_h = make(
            "[nx_h, E(n)] - C(b, n) * [nx, E(n)]",
            patterns, &scope
        ); 
    }

    return nx_f;
}

pub fn find_conter_exmaple(n: i32) -> bool {
    let patterns = &parser::load("./src/map");

    let mut scope = Scope(HashMap::new());

    let mut nx   = make("[E(1), E(2)]", patterns, &scope);
    let mut nx_f1 = make("[[E(1), E(2)], F(1)]", patterns, &scope);
    let mut nx_h1 = make("[[E(1), E(2)], H(1)]", patterns, &scope);
    let mut nx_f2 = make("[[E(1), E(2)], F(2)]", patterns, &scope);
    let mut nx_h2 = make("[[E(1), E(2)], H(2)]", patterns, &scope);
    let mut nx_f3 = make("[[E(1), E(2)], F(3)]", patterns, &scope);
    let mut nx_h3 = make("[[E(1), E(2)], H(3)]", patterns, &scope);

    for n in 2..n {
        println!("Checking {}", n);
        scope.0.insert("n".to_string(), Value::Number(n % 3 + 1));
        scope.0.insert("nx".to_string(), nx.clone());
        scope.0.insert("nx_f1".to_string(), nx_f1);
        scope.0.insert("nx_f2".to_string(), nx_f2);
        scope.0.insert("nx_f3".to_string(), nx_f3);
        scope.0.insert("nx_h1".to_string(), nx_h1);
        scope.0.insert("nx_h2".to_string(), nx_h2);
        scope.0.insert("nx_h3".to_string(), nx_h3);

        nx = brak(nx, e(n % 3 + 1)); 

        nx_f1 = if n & 3 + 1 == 1 {
            make("[nx_f1, E(n)] - nx_h1", patterns, &scope)
        } else {
            make("[nx_f1, E(n)]", patterns, &scope)
        };

        nx_f2 = if n & 3 + 1 == 2 {
            make("[nx_f2, E(n)] - nx_h2", patterns, &scope)
        } else {
            make("[nx_f2, E(n)]", patterns, &scope)
        };

        nx_f3 = if n & 3 + 1 == 3 {
            make("[nx_f3, E(n)] - nx_h3", patterns, &scope)
        } else {
            make("[nx_f3, E(n)]", patterns, &scope)
        };

        nx_h1 = make("[nx_h1, E(n)] - C(1, n) * [nx, E(n)]", patterns, &scope); 
        nx_h2 = make("[nx_h2, E(n)] - C(2, n) * [nx, E(n)]", patterns, &scope); 
        nx_h3 = make("[nx_h3, E(n)] - C(3, n) * [nx, E(n)]", patterns, &scope); 

        if nx_f1 == Value::zero() && nx_f2 == Value::zero() && nx_f3 == Value::zero()  {
            println!("ZERO!!! @ {}", n);
            return true;
        }
    }

    return false;
}

fn main() {
    v2::find_counter_example();

    let patterns = &parser::load("./src/map");
    let scope = &Scope(HashMap::new());
    // println!("\nn4_h1 = {}", make("[[[[E(1), E(2)], E(3)], E(1)], H(1)]", patterns, scope));
}