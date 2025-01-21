mod strategy_impl {
    #[derive(Debug)]
    pub struct A;

    #[derive(Debug)]
    pub struct B;

    #[derive(Debug)]
    pub struct C;
}

enum_inliner::enum_inline!(
    #[derive(Copy, Clone)]
    enum Strategy {
        A,
        B,
        C,
    }

    impl<const __VARIANT__: ident> Strategy {
        fn to_s(self) -> String {
            format!("{:?}", strategy_impl::__VARIANT__)
        }
    }
);

#[test]
fn compiles() {
    use Strategy::*;

    let cases = &[(A, "A"), (B, "B"), (C, "C")];
    for (strategy, expected) in cases {
        let actual = strategy.to_s();
        assert_eq!(actual, *expected);
    }
}
