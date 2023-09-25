#![feature(box_patterns)]
#![feature(extract_if)]

pub mod expr;
pub mod rule;

mod either;

#[cfg(test)]
mod tests {
    use super::*;
    use expr::Expr;
    use rule::CompositeRule;
    use rule::DefaultRule;
    use rule::NNFRule;
    use rule::CNFRule;
    use rule::DNFRule;

    #[test]
    fn test_expr() {
        let a = Expr::variable("a");
        let b = Expr::variable("b");
        let c = Expr::variable("c");
        let d = Expr::variable("d");
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

        let a = Expr::variable("a");
        let b = Expr::variable("b");
        let c = Expr::variable("c");
        let d = Expr::variable("d");
        let simplified = Expr::disjunction([
            Expr::conjunction([a, b, c]),
            Expr::negation(d)]
        );

        let a = Expr::variable("a");
        let b = Expr::variable("b");
        let c = Expr::variable("c");
        let d = Expr::variable("d");
        let cnf = Expr::conjunction([
            Expr::disjunction([a, Expr::negation(d.clone())]),
            Expr::disjunction([b, Expr::negation(d.clone())]),
            Expr::disjunction([c, Expr::negation(d.clone())]),
        ]);

        let a = Expr::variable("a");
        let b = Expr::variable("b");
        let c = Expr::variable("c");
        let d = Expr::variable("d");
        let dnf = Expr::disjunction([
            Expr::conjunction([a, b, c]),
            Expr::negation(d)]
        );

        assert_eq!(expr.clone().simplify(&CompositeRule(DefaultRule, NNFRule)), simplified);
        assert_eq!(expr.clone().simplify(&CompositeRule(CompositeRule(DefaultRule, NNFRule), CNFRule)), cnf);
        assert_eq!(expr.clone().simplify(&CompositeRule(CompositeRule(DefaultRule, NNFRule), DNFRule)), dnf);
    }
}
