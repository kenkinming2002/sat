#![feature(box_patterns)]
#![feature(extract_if)]

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Negation(Box<Expr>),
    Conjunction(Vec<Expr>),
    Disjunction(Vec<Expr>),
}

pub trait Rule {
    fn apply(&self, expr : Expr) -> Result<Expr, Expr>;
}

pub struct CompositeRule<L, R>(L, R);
impl<L: Rule, R: Rule> Rule for CompositeRule<L, R> {
    fn apply(&self, expr : Expr) -> Result<Expr, Expr> {
        let expr = match self.0.apply(expr) {
            Ok(expr)  => return Ok(expr),
            Err(expr) => expr,
        };
        let expr = match self.1.apply(expr) {
            Ok(expr)  => return Ok(expr),
            Err(expr) => expr,
        };
        Err(expr)
    }
}

pub struct DefaultRule;
impl Rule for DefaultRule {
    fn apply(&self, expr : Expr) -> Result<Expr, Expr> {
        match expr {
            Expr::Negation(box Expr::Negation(box expr)) => Ok(expr),
            Expr::Negation(box Expr::Conjunction(exprs)) => Ok(Expr::disjunction(exprs.into_iter().map(Expr::negation))),
            Expr::Negation(box Expr::Disjunction(exprs)) => Ok(Expr::conjunction(exprs.into_iter().map(Expr::negation))),
            Expr::Conjunction(exprs) if exprs.iter().any(|expr| matches!(expr, Expr::Conjunction(_))) => {
                let exprs = exprs.into_iter().flat_map(|expr| match expr {
                    Expr::Conjunction(exprs) => Either::Left(exprs.into_iter()),
                    expr                     => Either::Right(std::iter::once(expr)),
                });
                Ok(Expr::conjunction(exprs))
            },
            Expr::Disjunction(exprs) if exprs.iter().any(|expr| matches!(expr, Expr::Disjunction(_))) => {
                let exprs = exprs.into_iter().flat_map(|expr| match expr {
                    Expr::Disjunction(exprs) => Either::Left(exprs.into_iter()),
                    expr                     => Either::Right(std::iter::once(expr)),
                });
                Ok(Expr::disjunction(exprs))
            },
            expr => Err(expr),
        }
    }
}

pub struct CNFRule;
impl Rule for CNFRule {
    fn apply(&self, expr : Expr) -> Result<Expr, Expr> {
        match expr {
            Expr::Disjunction(exprs) if exprs.iter().any(|expr| matches!(expr, Expr::Conjunction(_))) => {
                let conjunctions = exprs.into_iter().map(|expr| match expr {
                    Expr::Conjunction(exprs) => Either::Left(exprs.into_iter()),
                    expr                     => Either::Right(std::iter::once(expr)),
                });
                let disjunctions = conjunctions.multi_cartesian_product().map(|exprs| Expr::Disjunction(exprs)).collect_vec();
                Ok(Expr::Conjunction(disjunctions))
            },
            expr => Err(expr),
        }
    }
}

pub struct DNFRule;
impl Rule for DNFRule {
    fn apply(&self, expr : Expr) -> Result<Expr, Expr> {
        match expr {
            Expr::Conjunction(exprs) if exprs.iter().any(|expr| matches!(expr, Expr::Disjunction(_))) => {
                let disjunctions = exprs.into_iter().map(|expr| match expr {
                    Expr::Disjunction(exprs) => Either::Left(exprs.into_iter()),
                    expr                     => Either::Right(std::iter::once(expr)),
                });
                let conjunctions = disjunctions.multi_cartesian_product().map(|exprs| Expr::Conjunction(exprs)).collect_vec();
                Ok(Expr::Disjunction(conjunctions))
            },
            expr => Err(expr),
        }
    }
}

impl Expr {
    pub fn unit<S: Into<String>>(s : S) -> Self { Self::Unit(s.into()) }
    pub fn negation(expr : Self) -> Self { Self::Negation(Box::new(expr)) }
    pub fn conjunction<I: IntoIterator<Item = Expr>>(exprs : I) -> Self { Self::Conjunction(Vec::from_iter(exprs)) }
    pub fn disjunction<I: IntoIterator<Item = Expr>>(exprs : I) -> Self { Self::Disjunction( Vec::from_iter(exprs)) }

    pub fn try_simplify<R : Rule>(self, rule : &R) -> Result<Self, Self> {
        let expr = match rule.apply(self) {
            Ok(expr) => return Ok(expr),
            Err(expr) => expr,
        };
        match expr {
            Self::Unit(ident) => Err(Self::Unit(ident)),
            Self::Negation(box expr) => match expr.try_simplify(rule) {
                Ok(expr)  => Ok(Self::negation(expr)),
                Err(expr) => Err(Self::negation(expr)),
            },
            Self::Conjunction(exprs) => {
                let exprs = exprs.into_iter().map(|expr| expr.try_simplify(rule)).collect_vec();
                let ok = exprs.iter().any(|expr| expr.is_ok());
                let exprs = exprs.into_iter().map(|expr| match expr {
                    Ok(expr) => expr,
                    Err(expr) => expr,
                }).collect_vec();
                if ok {
                    Ok(Self::conjunction(exprs))
                } else {
                    Err(Self::conjunction(exprs))
                }
            },
            Self::Disjunction(exprs) => {
                let exprs = exprs.into_iter().map(|expr| expr.try_simplify(rule)).collect_vec();
                let ok = exprs.iter().any(|expr| expr.is_ok());
                let exprs = exprs.into_iter().map(|expr| match expr {
                    Ok(expr) => expr,
                    Err(expr) => expr,
                }).collect_vec();
                if ok {
                    Ok(Self::disjunction(exprs))
                } else {
                    Err(Self::disjunction(exprs))
                }
            },
        }
    }

    pub fn simplify<R : Rule>(mut self, rule : &R) -> Self {
        loop {
            match self.try_simplify(rule) {
                Ok(expr)  => { self = expr; continue }
                Err(expr) => { self = expr; break self }
            }
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
        let expr = Expr::disjunction([
            Expr::conjunction([
                a.clone(),
                Expr::conjunction([
                    Expr::negation(Expr::negation(b.clone())),
                    c.clone()
                ]),
            ]),
            Expr::negation(Expr::negation(Expr::negation(d.clone()))),
        ]);

        let a = Expr::unit("a");
        let b = Expr::unit("b");
        let c = Expr::unit("c");
        let d = Expr::unit("d");
        let simplified = Expr::disjunction([
            Expr::conjunction([a, b, c]),
            Expr::negation(d)]
        );

        let a = Expr::unit("a");
        let b = Expr::unit("b");
        let c = Expr::unit("c");
        let d = Expr::unit("d");
        let cnf = Expr::conjunction([
            Expr::disjunction([a, Expr::negation(d.clone())]),
            Expr::disjunction([b, Expr::negation(d.clone())]),
            Expr::disjunction([c, Expr::negation(d.clone())]),
        ]);

        let a = Expr::unit("a");
        let b = Expr::unit("b");
        let c = Expr::unit("c");
        let d = Expr::unit("d");
        let dnf = Expr::disjunction([
            Expr::conjunction([a, b, c]),
            Expr::negation(d)]
        );

        assert_eq!(expr.clone().simplify(&DefaultRule), simplified);
        assert_eq!(expr.clone().simplify(&CompositeRule(DefaultRule, CNFRule)), cnf);
        assert_eq!(expr.clone().simplify(&CompositeRule(DefaultRule, DNFRule)), dnf);
    }
}
