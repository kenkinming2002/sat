use crate::expr::Expr;
use crate::either::Either;

use itertools::Itertools;

pub trait Rule {
    fn apply(&self, expr : Expr) -> Result<Expr, Expr>;
}

pub struct CompositeRule<L, R>(pub L, pub R);
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
                let disjunctions = conjunctions.multi_cartesian_product().map(Expr::Disjunction).collect_vec();
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
                let conjunctions = disjunctions.multi_cartesian_product().map(Expr::Conjunction).collect_vec();
                Ok(Expr::Disjunction(conjunctions))
            },
            expr => Err(expr),
        }
    }
}


