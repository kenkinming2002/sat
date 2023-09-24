#![feature(box_patterns)]
#![feature(extract_if)]

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
        ]);

        let a = Expr::unit("a");
        let b = Expr::unit("b");
        let c = Expr::unit("c");
        let d = Expr::unit("d");
        let rhs = Expr::or([
            Expr::and([a, b, c]),
            Expr::not(d)]
        );

        assert_eq!(lhs.simplify(), rhs);
    }
}
