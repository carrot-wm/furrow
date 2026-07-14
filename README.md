# furrow

A static site generator in a handful of std-only Rust files. No crates, no templates, no JavaScript in the output. Your site is a plain Rust binary that calls these modules; furrow is the row it grows in.

Grown for [carrotwm.org](https://carrotwm.org), the showcase site of the [carrot](https://github.com/carrot-wm/carrot) compositor, and extracted so other gardens can use the same row.

## The idea

Most generators hand you a templating language and a config format, then you fight both. furrow hands you five Rust modules and gets out of the way: pages are functions that return HTML strings, content is whatever you want it to be (markdown files, Rust data, SVG drawn in code), and the whole build is one binary you wrote yourself.

## Using it

Vendor `src/` into your project and declare the module:

```rust
mod furrow;

use furrow::html::{Page, Shell};
```

A single bare `rustc --edition 2024 -O src/main.rs -o mysite` compiles your whole site, no cargo required. If you do prefer cargo, furrow also works as a library dependency: the package is published as `furrow-ssg`, and its library target is still named `furrow`, so code reads the same either way.

The demo is a complete site in one file:

```sh
rustc --edition 2024 -O demo/main.rs -o demo-site
./demo-site serve      # build + host on http://127.0.0.1:8437
```

## The modules

- `md` renders a practical markdown subset to HTML: headings with anchor slugs, tables, fenced code, lists, blockquotes, inline code, bold, and links.
- `html` escapes text and wraps pages in a shell you define once (lang, og tags, and whatever head links your site self-hosts).
- `out` writes pages under pretty routes (`docs/tiling` becomes `docs/tiling/index.html`) and copies static trees.
- `serve` is a dev server in a page of std, with correct MIME types for resolved files.
- `github` fetches star counts, descriptions, and latest release asset names at build time through a curl subprocess, with baked fallbacks for offline builds.

## What it is not

There is no config file, no theme system, no plugin API, and no dependency ever. Furrow is currently extremely minimal - if your site needs a feature, it is your binary; write the function. PRs are encouraged.

## License

GPL-3.0, see [LICENSE](LICENSE).
