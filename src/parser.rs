use crate::value::Value;

#[derive(Clone, Copy, Debug)]
struct Parser<'a> {
    src: &'a str,
    index: usize,
}

impl<'a> Parser<'a> {
    fn set_index(&self, index: usize) -> Self {
        return Parser {
            src: self.src,
            index: std::cmp::min(self.index + index, self.src.len())
        }
    }

    fn starts_with(&self, func: impl FnOnce(char) -> bool) -> Option<Self> {
        if let Some((i, chr)) = self.src[self.index..].char_indices().next() {
            if func(chr) {
                return Some(self.set_index(i + 1));
            }
        }

        return None;
    }

    fn starts_with_char(&self, chr: char) -> Option<Self> {
        return self.starts_with(|c| c == chr);
    }

    fn next_while(&self, func: impl Fn(char) -> bool) -> Self {
        for (i, chr) in self.src[self.index..].char_indices() {
            if !func(chr) {
                return self.set_index(i); 
            }
        }

        return self.set_index(self.src.len())
    }

    fn skip_whitespace(&self) -> Self {
        return self.next_while(|chr| chr.is_whitespace());
    }
}

///////

#[derive(Debug, Clone)]
pub enum AST {
    Named(String),
    Value(Value),
    Braket(Box<AST>, Box<AST>),
    Add(Box<AST>, Box<AST>),
    Mul(Box<AST>, Box<AST>),
    Kind(String, String),
    Negative(Box<AST>),
    C(Box<AST>, Box<AST>),
}

impl AST {
    pub fn flip(&self) -> Option<Self> {
        if let AST::Braket(a, b) = self {
            return Some(AST::Braket(Box::new(*b.clone()), Box::new(*a.clone())))
        } else {
            return None;
        }
    }

    pub fn negate(&self) -> Self {
        return AST::Negative(Box::new(self.clone()));
    }
}

///////

fn parse_word(src: Parser) -> Option<(Parser, String)> {
    let start = src.index;

    let src = src.starts_with(|chr| chr.is_ascii_alphabetic())?;
    let src = src.next_while(|chr| chr.is_ascii_alphabetic());

    let end = src.index;

    return Some((src, src.src[start..end].to_string()));
}

fn parse_c(src: Parser) -> Option<(Parser, AST)> {
    let src = src.starts_with_char('C')?;
    let src = src.starts_with_char('(')?;
    let src = src.skip_whitespace();
    let (src, a) = parse_value(src)?;
    let src = src.starts_with_char(',')?;
    let src = src.skip_whitespace();
    let (src, b) = parse_value(src)?;
    let src = src.starts_with_char(')')?;

    return Some((src, AST::C(Box::new(a), Box::new(b))));
}

fn parse_named(src: Parser) -> Option<(Parser, AST)> {
    let (src, name) = parse_word(src)?;

    if let Some(src) = src.starts_with_char('(') {
        if let Some((src, value)) = parse_word(src) {
            let src = src.starts_with_char(')')?;

            return Some((src, AST::Kind(name, value)));
        }

        if let Some((src, value)) = parse_number(src) {
            let src = src.starts_with_char(')')?;

            return Some((src, AST::Value(Value::Kind(name, value))));
        }
    }

    return Some((src, AST::Named(name)));
}

fn parse_number(src: Parser) -> Option<(Parser, i32)> {
    let start = src.index;
    let src = src.next_while(|chr| chr.is_numeric());
    let end = src.index;

    if start == end {
        return None;
    }

    return Some((src, src.src[start..end].parse::<i32>().expect("Failed to parse number!")));
}

fn parse_number_value(src: Parser) -> Option<(Parser, AST)> {
    let (src, num) = parse_number(src)?;
    return Some((src, AST::Value(Value::Number(num))));
}

fn parse_braket(src: Parser) -> Option<(Parser, AST)> {
    let src = src.starts_with_char('[')?;
    let src = src.skip_whitespace();
    
    let (src, a) = parse_value(src)?;
    
    let src = src.starts_with_char(',')?;
    let src = src.skip_whitespace();
    
    let (src, b) = parse_value(src)?;

    let src = src.starts_with_char(']')?;
    let src = src.skip_whitespace();

    return Some((src, AST::Braket(Box::new(a), Box::new(b))));
}

fn parse_negative(src: Parser) -> Option<(Parser, AST)> {
    let src = src.starts_with_char('-')?;
    let src = src.skip_whitespace();
    let (src, value) = parse_value(src)?;
    return Some((src, AST::Negative(Box::new(value))));
}

fn parse_paren(src: Parser) -> Option<(Parser, AST)> {
    let src = src.starts_with_char('(')?;
    let src = src.skip_whitespace();
    let (src, value) = parse_value(src)?;
    let src = src.starts_with_char(')')?;
    return Some((src, value));
}

fn parse_value(src: Parser) -> Option<(Parser, AST)> {
    let (src, a) = parse_paren(src)
        .or_else(|| parse_c(src))
        .or_else(|| parse_named(src))
        .or_else(|| parse_number_value(src))
        .or_else(|| parse_braket(src))
        .or_else(|| parse_negative(src))?;

    let src = src.skip_whitespace();

    if let Some(src) = src.starts_with_char('+') {
        let src = src.skip_whitespace();

        let (src, b) = parse_value(src)?;

        return Some((src, AST::Add(Box::new(a), Box::new(b))))
    }

    if let Some(src) = src.starts_with_char('-') {
        let src = src.skip_whitespace();

        let (src, b) = parse_value(src)?;

        return Some((src, AST::Add(Box::new(a), Box::new(AST::Negative(Box::new(b))))))
    }

    if let Some(src) = src.starts_with_char('*') {
        let src = src.skip_whitespace();

        let (src, b) = parse_value(src)?;

        return Some((src, AST::Mul(Box::new(a), Box::new(b))))
    }

    return Some((src, a));
}

pub fn parse(src: &str) -> AST {
    if let Some((_, ast)) = parse_value(Parser { src, index: 0 }) {
        return ast;
    }

    panic!("Failed to parse {src}!");
}

pub fn load(path:  &str) -> Vec<(AST, AST)> {
    std::fs::read_to_string(path)
        .expect(&format!("Could not find source file {path}!"))
        .lines()
        .map(|line| line.trim())
        .filter(|line| line.len() != 0)
        .map(|line| {
            if let [pattern_src, expr_src] = line.split('=').collect::<Vec<&str>>()[..] {
                let pattern = parse(pattern_src.trim());
                let expr = parse(expr_src.trim());
                return (pattern, expr);
            } else {
                panic!("Invalid expression {line}, could not find equal statment!");
            }
        })
        .collect::<Vec<(AST, AST)>>()
}