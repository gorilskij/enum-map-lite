# enum-map-lite-derive

Derive macro for the `Enum` trait of
[`enum-map-lite`](https://crates.io/crates/enum-map-lite). You normally don't
depend on this crate directly — `enum-map-lite` re-exports the derive.

`#[derive(Enum)]` generates a discriminant-based `into_usize` (one slot per
variant, fields ignored) and the backing-array impl. It works on any enum.

## License

Licensed under either of Apache License, Version 2.0
([LICENSE-APACHE](LICENSE-APACHE)) or MIT license ([LICENSE-MIT](LICENSE-MIT))
at your option.
