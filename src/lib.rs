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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr() {
        let a = Expr::unit("a");
        let b = Expr::unit("b");
        let c = Expr::and([a, b]);
        let d = Expr::unit("d");
        let e = Expr::not(d);
        let _ = Expr::or([c, e]);
    }
}
