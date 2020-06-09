## Butcher

An easy way to interact with `Cow`ed structs and enums.

## Disclaimer

This crate is still in early steps of developpments. It should not be used in
production.

## Concept

This crate aims to allow allow simple destructuring (for `struct`s), pattern
matching (for `enum`s and `struct`s) and iteration (for `enum`s and `struct`s
that implement it).

### Destructuring

TODO

### Pattern matching

See [this gist](https://gist.github.com/5bb57b8bf4bfc08758d9cb557e1fdbfe).

### Iteration

This crate provide a `CowIter` type, which allows to write `Cow` fiendly
iterators. See this example:

```rust
use std::borrow:Cow;
use butcher::iterator::CowIter;

fn print_numbers(elems: Cow<[u32]>) {
    let mut iter = CowIter::from_cow(elems)

    for element in iter {
        // The type of element is Cow<u32>
        println!("{:?}", element);
    }
}
```

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

