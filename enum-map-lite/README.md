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

## `enum_map!` keys

Keys are **patterns**, not values — data-carrying variants never need a
constructed payload. Only the variant matters; fields must be `_` or `..`, never
bindings or value matching:

```rust
use enum_map_lite::{enum_map, Enum, EnumMap};

#[derive(Enum, Clone, Copy)]
enum Event {
    Tick,                       // unit
    Key(char),                  // tuple
    Click(u32, u32),            // tuple
    Resize { w: u32, h: u32 },  // struct
    Scroll { delta: i32 },      // struct
}

let m: EnumMap<Event, &str> = enum_map! {
    Event::Tick => "tick",
    Event::Key(_) => "key",                 // tuple, correct arity
    Event::Click(..) => "click",            // tuple, elided
    Event::Resize { .. } => "resize",       // struct, elided
    Event::Scroll { delta: _ } => "scroll", // struct, named field as `_`
};
assert_eq!(m[Event::Click(10, 20)], "click"); // any payload → same slot
```

An optional `_ => default` catch-all fills every unlisted variant at construction
time. Without it the key list must be exhaustive — a missing variant panics at
construction.

## Generic enums

Generic enums work too: the shadow mirror drops the type parameters, so the
derive is unconditional. Spell the parameters on the key with turbofish so the
map's key type can be inferred:

```rust
use enum_map_lite::{enum_map, Enum, EnumMap};

#[derive(Enum, Clone, Copy)]
enum Slot<T> {
    Empty,
    Filled(T),
    Reserved { by: u32 },
}

let m: EnumMap<Slot<String>, u8> = enum_map! {
    Slot::<String>::Filled(..) => 1,
    _ => 0,
};
assert_eq!(m[Slot::Filled("hi".to_string())], 1);
assert_eq!(m[Slot::<String>::Empty], 0);
```

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
