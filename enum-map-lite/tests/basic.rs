use enum_map_lite::{enum_map, Enum, EnumMap};

#[derive(Enum, Clone, Copy, Debug, PartialEq)]
enum Dir {
    North,
    East,
    South,
    West,
}

#[test]
fn default_form_fills_unlisted() {
    let m: EnumMap<Dir, i32> = enum_map! {
        Dir::North => 1,
        Dir::South => 3,
        _ => 0,
    };
    assert_eq!(m[Dir::North], 1);
    assert_eq!(m[Dir::East], 0);
    assert_eq!(m[Dir::South], 3);
    assert_eq!(m[Dir::West], 0);
}

#[test]
fn exhaustive_form() {
    let m: EnumMap<Dir, &str> = enum_map! {
        Dir::North => "n",
        Dir::East => "e",
        Dir::South => "s",
        Dir::West => "w",
    };
    assert_eq!(m[Dir::East], "e");
    assert_eq!(m[Dir::West], "w");
}

#[test]
fn only_default() {
    let m: EnumMap<Dir, u8> = enum_map! { _ => 7 };
    assert!(m.values().all(|&v| v == 7));
}

#[test]
fn index_mut_and_get() {
    let mut m: EnumMap<Dir, i32> = enum_map! { _ => 0 };
    m[Dir::East] = 42;
    *m.get_mut(Dir::West) = -1;
    assert_eq!(*m.get(Dir::East), 42);
    assert_eq!(m[Dir::West], -1);
}

#[test]
fn value_iteration() {
    let m: EnumMap<Dir, i32> = enum_map! {
        Dir::North => 1,
        Dir::East => 2,
        Dir::South => 3,
        Dir::West => 4,
    };
    assert_eq!(m.values().sum::<i32>(), 10);
    assert_eq!((&m).into_iter().copied().sum::<i32>(), 10);
    assert_eq!(m.into_values().sum::<i32>(), 10);
}

#[test]
fn map_preserves_keys() {
    let m: EnumMap<Dir, i32> = enum_map! {
        Dir::North => 1,
        Dir::East => 2,
        Dir::South => 3,
        Dir::West => 4,
    };
    let doubled = m.map(|v| v * 10);
    assert_eq!(doubled[Dir::North], 10);
    assert_eq!(doubled[Dir::West], 40);
}

// Fields are ignored: one slot per variant, keyed on the discriminant.
#[derive(Enum, Clone, Copy)]
#[allow(dead_code)] // payload fields exist only to exercise field-carrying variants
enum Seg {
    Normal,
    Eaten { original_food: u32, food_left: u32 },
    Crashed,
    BlackHole(bool),
}

#[test]
fn field_variants_key_on_discriminant() {
    assert_eq!(Seg::LENGTH, 4);
    let mut m: EnumMap<Seg, i32> = enum_map! {
        Seg::Eaten { original_food: 0, food_left: 0 } => 1,
        _ => 0,
    };
    // any payload maps to the same slot as the pattern above
    assert_eq!(m[Seg::Eaten { original_food: 999, food_left: 7 }], 1);
    assert_eq!(m[Seg::Normal], 0);
    assert_eq!(m[Seg::BlackHole(true)], 0);
    m[Seg::BlackHole(false)] = 42;
    assert_eq!(m[Seg::BlackHole(true)], 42); // field ignored on lookup too
}

#[test]
fn copy_and_clone() {
    let m: EnumMap<Dir, i32> = enum_map! { _ => 5 };
    let c = m; // Copy
    let d = m.clone();
    assert_eq!(c[Dir::North], 5);
    assert_eq!(d[Dir::South], 5);
    assert_eq!(Dir::LENGTH, 4);
}
