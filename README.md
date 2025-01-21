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

See the complete code in this [simple example test].

[simple example test]:
  https://github.com/lffg/enum_inliner/blob/main/tests/compiles.rs

## Use cases

Use cases aren't vast; but sometimes this may come in handy.

One of such use cases is when working with traits that aren't
object-safe/dyn-safe (cannot be used with `dyn`), but still require dynamic
dispatch at runtime. In these situations, one can create an enum with variants
representing each trait implementation, which is used to perform the dispatch
based on the enum "tags", which can pass through runtime.

## Why a proc macro?

Even though this kind of functionality [can be implemented][decl-impl] using a
declarative macro, the `macro_rules!` implementation becomes _extremely_
convoluted. User-level error messages also get degraded.

[decl-impl]: https://gist.github.com/lffg/94cbb0172a035075e29c46f2f1f31908

## License

MIT license.
