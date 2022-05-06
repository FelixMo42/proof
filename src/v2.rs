///        
#[derive(Clone, Eq, PartialEq)]
struct Term {
    n: i32,
    es: Vec<i32>,
}

impl Term {
    fn new(n: i32, es: Vec<i32>) -> Term {
        return Term { n, es };
    }
}

///
#[derive(Clone, Eq, PartialEq)]
struct Value {
    terms: Vec<Term>
}

impl Value {
    fn new(terms: Vec<Term>) -> Value {
        Value { terms }
    }
}

impl Value {
    fn braket(mut self, e: i32) -> Value {
        for term in &mut self.terms {
            term.es.push(e);

            if term.es.len() == 2 {
                if term.es[1] > term.es[0] {
                    term.es[1] = term.es[0];
                    term.es[0] = e;
                    term.n = -term.n;
                }
            }
        }

        return self;
    }

    fn times(mut self, n: i32) -> Value {
        for term in &mut self.terms {
            term.n *= n; 
        }

        return self;
    }

    fn add(mut self, b: Value) -> Value {
        b.terms
            .into_iter()
            .for_each(|term_b| {
                for term_a in &mut self.terms {
                    if term_a.es == term_b.es {
                        term_a.n += term_b.n;
                        return;
                    }
                }
               
                self.terms.push(term_b);
            });

        for i in (0..self.terms.len()).rev() {
            if self.terms[i].n == 0 {
                self.terms.remove(i);
            }
        }

        return self;
    }

    fn equals_zero(&self) -> bool {
        return self.terms.len() == 0;
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for term in &self.terms {
            write!(f, "{} * ", term.n)?;

            for _ in 1..term.es.len() {
                write!(f, "[")?;
            }

            for i in 0..term.es.len() {
                if i == 0 {
                    write!(f, "E({})", term.es[i])?;
                } else {
                    write!(f, ", E({})]", term.es[i])?;
                }
            }


            write!(f, " + ")?;
        }

        return Ok(());
    }
}

///
const C: [[i32; 3]; 3] = [
    [  2, -1, -1 ],
    [ -1,  2, -2 ],
    [ -1, -1,  2 ]
];

fn c(x: i32, y: i32) -> i32 {
    return C[x as usize][y as usize];
}


///
pub fn find_counter_example() {
    // nx = [E(1), E(2)]
    let mut nx = Value::new(vec![Term::new(-1, vec![ 2, 1 ])]);

    // nx_f1 = [[E(1), E(2)], F(1)] =
    let mut nx_f1 = Value::new(vec![Term::new(-1, vec![ 2 ])]);
    // nx_f2 = [[E(1), E(2)], F(1)]
    let mut nx_f2 = Value::new(vec![Term::new(0, vec![ 1 ])]);
    // nx_f3 = [[E(1), E(2)], F(1)]
    let mut nx_f3 = Value::new(vec![]);

    // nx_h1 = [[E(1), E(2)], H(1)]
    let mut nx_h1 = Value::new(vec![Term::new(1, vec![ 2, 1 ])]);
    // nx_h2 = [[E(1), E(2)], H(1)]
    let mut nx_h2 = Value::new(vec![Term::new(1, vec![ 2, 1 ])]);
    // nx_h3 = [[E(1), E(2)], H(2)]
    let mut nx_h3 = Value::new(vec![Term::new(-2, vec![ 2, 1 ])]);

    for n in 2.. {
        if n % 500 == 0 {
            println!("Checking {}", n);
        }
        // println!("n{n} = {nx}");
        // println!("n{n}_h1 = {nx_h1}");

        nx_f1 = if n % 3 + 1 == 1 {
            // [nx_f1, E(n % 3 + 1)] - nx_h1
            nx_f1.braket(n % 3 + 1).add(nx_h1.clone().times(-1))
        } else {
            // [nx_f1, E(n % 3 + 1)]
            nx_f1.braket(n % 3 + 1)
        };

        nx_f2 = if n % 3 + 1 == 2 {
            // [nx_f1, E(n % 3 + 1)] - nx_h1
            nx_f2.braket(n % 3 + 1).add(nx_h2.clone().times(-1))
        } else {
            // [nx_f1, E(n % 3 + 1)]
            nx_f2.braket(n % 3 + 1)
        };

        nx_f3 = if n % 3 + 1 == 1 {
            // [nx_f1, E(n % 3 + 1)] - nx_h1
            nx_f3.braket(n % 3 + 1).add(nx_h3.clone().times(-1))
        } else {
            // [nx_f1, E(n % 3 + 1)]
            nx_f3.braket(n % 3 + 1)
        };

        nx_h1 = nx_h1.braket(n % 3 + 1).add(nx.clone().braket(n % 3 + 1).times(-c(0, n % 3)));
        nx_h2 = nx_h2.braket(n % 3 + 1).add(nx.clone().braket(n % 3 + 1).times(-c(1, n % 3)));
        nx_h3 = nx_h3.braket(n % 3 + 1).add(nx.clone().braket(n % 3 + 1).times(-c(2, n % 3)));

        // nx = [nx, E(n % 3 + 1)]
        nx = nx.braket(n % 3 + 1);

        if nx_f1.equals_zero() && nx_f2.equals_zero() && nx_f3.equals_zero() {
            println!("Found zero @ {n}!!");
            return;
        }
    }
}