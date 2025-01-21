# `enum_inliner`

Takes this:

```rs
mod strategy_impl { ... }

enum_inliner::enum_inline!(
    #[derive(Copy, Clone)]
    enum Strategy {
        A,
        B,
        C,
    }

    impl<const __VARIANT__: ident> Strategy {
        fn dbg_s(self) -> String {
            format!("{:?}", strategy_impl::__VARIANT__)
        }
    }
);
```

And expands each function inside the provided `impl` block by "copying" its body
— the *template* — into the `match` arms generated for the corresponding enum.
One would get this:

```rs
mod strategy_impl { ... }

#[derive(Copy, Clone)]
enum Strategy {
    A,
    B,
    C,
}

impl Strategy {
    fn dbg_s(self) -> String {
        match self {
            Self::A => format!("{:?}", strategy_impl::A),
            Self::B => format!("{:?}", strategy_impl::B),
            Self::C => format!("{:?}", strategy_impl::C),
        }
    }
}
```

See the complete code in this [example test].

[example test]: https://github.com/lffg/enum_inliner/blob/main/tests/compiles.rs

Use cases aren't vast; but sometimes this may come in handy.
