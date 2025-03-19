# fn-item.rs

<!-- cargo-rdme start -->

Helpers for working with closures that don’t capture any variables.

The [`fn_item!`] macro makes a closure with no captures,
and can be accepted into functions with [`ImplFnItem!`].

This is useful for dealing with function pointers in a more composable way.

## Examples

```rust
fn make_fn_ptr<F>(_: ImplFnItem![F: for<'a> Fn(&'a str) -> u32]) -> fn(&str) -> u32
where
    F: for<'a> FnItem<(&'a str,), u32>
{
    |s| F::call((s,))
}

let fn_ptr = make_fn_ptr(fn_item!(|s| s.parse::<u32>().unwrap()));
assert_eq!(fn_ptr("4115"), 4115);

use fn_item::FnItem;
use fn_item::ImplFnItem;
use fn_item::fn_item;
```

If you don’t want to add a generic parameter to your outer function,
you can use an inner function instead:

```rust
fn make_fn_ptr((f, ..): ImplFnItem![for<'a> Fn(&'a str) -> u32]) -> fn(&str) -> u32 {
    fn inner<F: for<'a> FnItem<(&'a str,), u32>>(_: F) -> fn(&str) -> u32 {
        |s| F::call((s,))
    }
    inner(f)
}
```

<!-- cargo-rdme end -->

## License

MIT.
