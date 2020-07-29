## Butcher

[![Build Status][actions-badge]][actions-url]
[![Latest Version][version-badge]][version-url]
[![Rust Documentation][docs-badge]][docs-url]

[actions-badge]: https://github.com/scileo/butcher/workflows/Continuous%20integration/badge.svg
[actions-url]: https://github.com/scileo/butcher/actions?query=workflow%3A%22Continuous+integration%22
[version-badge]: https://img.shields.io/crates/v/butcher.svg
[version-url]: https://crates.io/crates/butcher
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg
[docs-url]: https://docs.rs/butcher

An easy way to interact with `Cow`ed structs and enums.

## Disclaimer

This crate is still in early steps of developpments. It should not be used in
production.

## Concept

The standard library has a [clone on write type][cow], which allows to work
with data with either owned or borrowed data, without any distinction. However,
this can lead to a lot of code duplication in some situations.

[cow]: https://doc.rust-lang.org/std/borrow/enum.Cow.html

This crate aims to allow allow simple destructuring, pattern matching and
iteration over data wrapped in `Cow`.

### Destructuring

The `Butcher` trait can be derived on structs. Destructuring is then made easy:

```rust
use std::borrow::Cow;
use butcher::Butcher;

#[derive(Butcher, Clone)]
struct MyNumberList {
    val: u32,
    next: Option<Box<MyNumberList>>,
}

fn get_elem(i: Cow<MyNumberList>) -> Cow<u32> {
    let ButcheredMyNumberList { val, .. } = Butcher::butcher(i);

    val
}
```

### Pattern matching

The `Butcher` trait can also be derived on enums. This allows, for example:

```rust
use butcher::Butcher;
use std::borrow::Cow;

#[derive(Butcher, Clone)]
enum WebEvent {
    PageLoad,
    KeyPress(char),
    Paste(String),
    // or c-like structures.
    Click { x: i64, y: i64 },
}

fn print_action(i: Cow<WebEvent>) {
    match WebEvent::butcher(i) {
        ButcheredWebEvent::PageLoad => { /* ... */ },
        ButcheredWebEvent::KeyPress(key) => { /* ... */ },
        ButcheredWebEvent::Paste(pasted) => { /* ... */ },
        ButcheredWebEvent::Click { x, y } => { /* ... */ },
    }
}
```

The fields in each variant will be `Cow<T>` by default. This can be configured.
See the documentation for more information.


### Iteration

This crate provide a `CowIter` type, which allows to write `Cow` fiendly
iterators. See this example:

```rust
use std::borrow::Cow;
use butcher::iterator::CowIter;

fn print_numbers(elems: Cow<[u32]>) {
    let mut iter = CowIter::from(elems);

    for element in iter {
        // The type of element is Cow<u32>
        println!("{:?}", element);
    }
}
```

### Minimum Supported Rust Version

This crate compiles in rust 1.42 and older. Upgrading MSRV is a breaking change.
CI is set up so that it guarantees that the crate compiles and tests pass on
both 1.42 and stable rust.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

