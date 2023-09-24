#![feature(box_patterns)]
#![feature(extract_if)]

use itertools::Itertools;

enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Iterator for Either<L, R>
    where L: Iterator,
          R: Iterator<Item = L::Item>
{
    type Item = L::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Left(l)  => l.next(),
            Self::Right(r) => r.next(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Unit(String),
    Not(Box<Expr>),
    And(Vec<Expr>),
    Or(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    ident  : String,
    negate : bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    literals : Vec<Literal>,
}

#[derive(Debug, Clone, PartialEq, Eq)] pub struct ConjunctiveNormalForm { clauses : Vec<Clause> }
#[derive(Debug, Clone, PartialEq, Eq)] pub struct DisjunctiveNormalForm { clauses : Vec<Clause> }

impl Literal {
    fn negate(self) -> Self {
        Self { ident : self.ident, negate : !self.negate }
    }
}

impl Expr {
    pub fn unit<S: Into<String>>(s : S) -> Self { Self::Unit(s.into()) }
    pub fn not(expr : Self) -> Self { Self::Not(Box::new(expr)) }
    pub fn and<I: IntoIterator<Item = Expr>>(exprs : I) -> Self { Self::And(Vec::from_iter(exprs)) }
    pub fn or <I: IntoIterator<Item = Expr>>(exprs : I) -> Self { Self::Or( Vec::from_iter(exprs)) }

    pub fn simplify(self) -> Self {
        match self {
            Expr::Not(box Expr::Not(box expr)) => expr.simplify(),
            Expr::Not(box Expr::And(exprs)) => Expr::or (exprs.into_iter().map(Expr::not).map(Expr::simplify)).simplify(),
            Expr::Not(box Expr::Or (exprs)) => Expr::and(exprs.into_iter().map(Expr::not).map(Expr::simplify)).simplify(),
            Expr::And(exprs) => Expr::and(exprs.into_iter().flat_map(|expr| match expr { Expr::And(exprs) => Either::Left(exprs.into_iter().map(Expr::simplify)), expr => Either::Right(std::iter::once(expr.simplify())), })),
            Expr::Or (exprs) => Expr::or (exprs.into_iter().flat_map(|expr| match expr { Expr::Or (exprs) => Either::Left(exprs.into_iter().map(Expr::simplify)), expr => Either::Right(std::iter::once(expr.simplify())), })),
            expr => expr,
        }
    }

    pub fn to_cnf(&self) -> ConjunctiveNormalForm {
        match self {
            Self::Unit(ident) => ConjunctiveNormalForm { clauses : vec![Clause { literals : vec![Literal { ident : ident.clone(), negate : false }] }] },
            Self::Not(expr) => {
                let cnf = expr.to_cnf();
                let literals = cnf.clauses.into_iter().map(|clause| clause.literals.into_iter()); // Iterator of Iterator of Literal
                let literals = literals.map(|literals| literals.map(Literal::negate));            // Iterator of Iterator of Literal
                let literals = literals.multi_cartesian_product();
                ConjunctiveNormalForm { clauses : literals.map(|literals| Clause { literals }).collect_vec() }
            },
            Self::And(exprs) => {
                let cnfs = exprs.into_iter().map(|expr| expr.to_cnf());
                let clauses = cnfs.flat_map(|cnf| cnf.clauses).collect_vec();
                ConjunctiveNormalForm { clauses }
            },
            Self::Or(exprs) => {
                let cnfs = exprs.into_iter().map(|expr| expr.to_cnf());
                let clauses = cnfs.map(|cnf| cnf.clauses);
                let clauses = clauses.multi_cartesian_product(); // Iterator of Iterator of Clause
                let clauses = clauses.map(|clauses| Clause { literals : clauses.into_iter().flat_map(|clause| clause.literals).collect_vec() }).collect_vec();
                ConjunctiveNormalForm { clauses }
            },
        }
    }

    pub fn to_dnf(&self) -> DisjunctiveNormalForm {
        match self {
            Self::Unit(ident) => DisjunctiveNormalForm { clauses : vec![Clause { literals : vec![Literal { ident : ident.clone(), negate : false }] }] },
            Self::Not(expr) => {
                let dnf = expr.to_dnf();
                let literals = dnf.clauses.into_iter().map(|clause| clause.literals.into_iter()); // Iterator of Iterator of Literal
                let literals = literals.map(|literals| literals.map(Literal::negate));            // Iterator of Iterator of Literal
                let literals = literals.multi_cartesian_product();
                DisjunctiveNormalForm { clauses : literals.map(|literals| Clause { literals }).collect_vec() }
            },
            Self::And(exprs) => {
                let dnfs = exprs.into_iter().map(|expr| expr.to_dnf());
                let clauses = dnfs.map(|dnf| dnf.clauses);
                let clauses = clauses.multi_cartesian_product(); // Iterator of Iterator of Clause
                let clauses = clauses.map(|clauses| Clause { literals : clauses.into_iter().flat_map(|clause| clause.literals).collect_vec() }).collect_vec();
                DisjunctiveNormalForm { clauses }
            },
            Self::Or(exprs) => {
                let dnfs = exprs.into_iter().map(|expr| expr.to_dnf());
                let clauses = dnfs.flat_map(|dnf| dnf.clauses).collect_vec();
                DisjunctiveNormalForm { clauses }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr() {
        let a = Expr::unit("a");
        let b = Expr::unit("b");
        let c = Expr::unit("c");
        let d = Expr::unit("d");
        let lhs = Expr::or([
            Expr::and([
                a.clone(),
                Expr::and([
                    Expr::not(Expr::not(b.clone())),
                    c.clone()
                ]),
            ]),
            Expr::not(Expr::not(Expr::not(d.clone()))),
        ]).simplify();

        let a = Expr::unit("a");
        let b = Expr::unit("b");
        let c = Expr::unit("c");
        let d = Expr::unit("d");
        let rhs = Expr::or([
            Expr::and([a, b, c]),
            Expr::not(d)]
        );

        assert_eq!(lhs.simplify(), rhs);
        assert_eq!(rhs.to_cnf(), ConjunctiveNormalForm { clauses : vec![
            Clause { literals : vec![Literal { ident : "a".into(), negate : false }, Literal { ident : "d".into(), negate : true }] },
            Clause { literals : vec![Literal { ident : "b".into(), negate : false }, Literal { ident : "d".into(), negate : true }] },
            Clause { literals : vec![Literal { ident : "c".into(), negate : false }, Literal { ident : "d".into(), negate : true }] },
        ]});
        assert_eq!(rhs.to_dnf(), DisjunctiveNormalForm { clauses : vec![
            Clause { literals : vec![Literal { ident : "a".into(), negate : false }, Literal { ident : "b".into(), negate : false }, Literal { ident : "c".into(), negate : false }] },
            Clause { literals : vec![Literal { ident : "d".into(), negate : true }] },
        ]});
    }
}
