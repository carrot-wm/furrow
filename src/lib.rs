// furrow: a static site generator in a handful of std-only Rust files.
// no crates, no templates, no javascript in the output. your site is a
// plain Rust binary that calls these modules; furrow is the row it grows in.
//
// two ways to use it:
//   - vendor src/ into your project and declare `mod furrow;`
//     (a single bare `rustc --edition 2024 -O src/main.rs` still compiles)
//   - or depend on it as a cargo library
//
// grown for carrotwm.org, extracted so other gardens can use the same row.

pub mod github;
pub mod html;
pub mod md;
pub mod out;
pub mod serve;
