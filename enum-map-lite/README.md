# enum-map-lite

A stripped-down enum-keyed map backed by a fixed array.

An `EnumMap<K, V>` is a `[V; K::LENGTH]` indexed by a key enum `K`. Unlike the
[`enum-map`](https://crates.io/crates/enum-map) crate, the `Enum` trait here
deliberately has **no `from_usize`** — the map is a one-way `enum -> value`
index. The consequences of that choice:

- `#[derive(Enum)]` works on **any** enum, including data-carrying variants,
  keying purely on the **discriminant** (one slot per variant, fields ignored).
  No requirement that field types themselves implement `Enum`.
- There is **no key or pair iteration** (you can't recover a key from an index).
  Value iteration is fully supported.
- `#![no_std]`, no allocation. `EnumMap` is `Copy` when `V: Copy`.

```rust
use enum_map_lite::{enum_map, Enum, EnumMap};

#[derive(Enum, Clone, Copy)]
enum Color { Red, Green, Blue }

let mut m: EnumMap<Color, i32> = enum_map! {
    Color::Red => 1,
    _ => 0,          // fills Green and Blue at construction time
};
assert_eq!(m[Color::Red], 1);
m[Color::Blue] = 5;
assert_eq!(m.values().copied().sum::<i32>(), 6);
```

Keys are **patterns**, not values, so data-carrying variants never need a
constructed payload — they key on the discriminant:

```rust
use enum_map_lite::{enum_map, Enum, EnumMap};

#[derive(Enum, Clone, Copy)]
enum Seg { Normal, Eaten { food: u32 }, Crashed }

let m: EnumMap<Seg, &str> = enum_map! {
    Seg::Eaten { .. } => "eaten",
    _ => "other",
};
assert_eq!(m[Seg::Eaten { food: 999 }], "eaten"); // any payload → same slot
```

## `enum_map!` keys

Each key is a variant **pattern**; only the variant matters (fields are ignored).
Any variant shape works, and fields may only be `_` or `..` — never bindings or
value matching:

```text
Variant                 // unit or any variant by bare path
Variant()               // empty tuple
Variant(_, _)           // tuple, correct arity
Variant(..)             // tuple, elided
Variant { a: _, b: _ }  // struct, all fields as `_`
Variant { a: _, .. }    // struct, some fields + `..`
Variant { .. }          // struct, all elided
```

Plus an optional `_ => default` catch-all. Two forms:

- **with `_ => default`** — fills every unlisted variant at build time.
- **exhaustive, no catch-all** — panics at construction if a variant was missed.

Generic enums work too; spell the type parameters on the key with turbofish
(`Generic::<T>::Variant(..) => ...`) so the map's key type can be inferred.

## When to use `enum-map` instead

If you need key/pair iteration, or you want data-carrying variants to expand
into one slot per field-value combination, use the full
[`enum-map`](https://crates.io/crates/enum-map) crate. `enum-map-lite` trades
those away for discriminant-only keying that works on any enum.

## Credit

API and design are inspired by [`enum-map`](https://crates.io/crates/enum-map)
by Luna Borowska. This is an independent implementation; no source was copied.
Both crates are licensed `MIT OR Apache-2.0`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
