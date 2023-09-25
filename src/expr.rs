use crate::rule::Rule;

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Constant(bool),
    Variable(String),
    Negation(Box<Expr>),
    Conjunction(Vec<Expr>),
    Disjunction(Vec<Expr>),
}

impl Expr {
    pub fn unit<S: Into<String>>(s : S) -> Self { Self::Variable(s.into()) }
    pub fn negation(expr : Self) -> Self { Self::Negation(Box::new(expr)) }
    pub fn conjunction<I: IntoIterator<Item = Expr>>(exprs : I) -> Self { Self::Conjunction(Vec::from_iter(exprs)) }
    pub fn disjunction<I: IntoIterator<Item = Expr>>(exprs : I) -> Self { Self::Disjunction( Vec::from_iter(exprs)) }

    pub fn try_simplify<R : Rule>(self, rule : &R) -> Result<Self, Self> {
        let expr = match rule.apply(self) {
            Ok(expr) => return Ok(expr),
            Err(expr) => expr,
        };
        match expr {
            Self::Constant(value) => Err(Self::Constant(value)),
            Self::Variable(ident) => Err(Self::Variable(ident)),
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

