# Font [![Version][version-img]][version-url] [![Status][status-img]][status-url]

The package provides a font toolbox.

## [Documentation][doc]

## Example

```rust
use font::{File, Operation};

let path = "SourceSerifPro-Regular.otf";
let file = File::open(path).unwrap();
let glyph = file[0].draw('&').unwrap().unwrap();

for operation in glyph.iter() {
    match operation {
        &Operation::Move(..) => println!("Move!"),
        &Operation::Line(..) => println!("Line!"),
        &Operation::Curve(..) => println!("Curve!"),
    }
}
```

## Contribution

Your contribution is highly appreciated. Do not hesitate to open an issue or a
pull request. Note that any contribution submitted for inclusion in the project
will be licensed according to the terms given in [LICENSE.md](LICENSE.md).

[doc]: https://bodoni.github.io/font
[status-img]: https://travis-ci.org/bodoni/font.svg?branch=master
[status-url]: https://travis-ci.org/bodoni/font
[version-img]: https://img.shields.io/crates/v/font.svg
[version-url]: https://crates.io/crates/font
