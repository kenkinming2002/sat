use crate::rule::Rule;

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<T> {
    Variable(T),
    Constant(bool),
    Negation(Box<Self>),
    Conjunction(Vec<Self>),
    Disjunction(Vec<Self>),
}

impl<T> Expr<T> {
    pub fn variable(name : T) -> Self { Self::Variable(name) }
    pub fn negation(expr : Self) -> Self { Self::Negation(Box::new(expr)) }
    pub fn conjunction<I: IntoIterator<Item = Self>>(exprs : I) -> Self { Self::Conjunction(Vec::from_iter(exprs)) }
    pub fn disjunction<I: IntoIterator<Item = Self>>(exprs : I) -> Self { Self::Disjunction( Vec::from_iter(exprs)) }

    pub fn try_simplify<R : Rule<T>>(self, rule : &R) -> Result<Self, Self> {
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

    pub fn simplify<R : Rule<T>>(mut self, rule : &R) -> Self {
        loop {
            match self.try_simplify(rule) {
                Ok(expr)  => { self = expr; continue }
                Err(expr) => { self = expr; break self }
            }
        }
    }
}

