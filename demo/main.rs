// a whole site in one file: the vendor pattern, demonstrated.
//
//   rustc --edition 2024 -O demo/main.rs -o demo-site
//   ./demo-site serve        # build + host on http://127.0.0.1:8437
//
// your real site looks like this with more pages: markdown (or plain rust
// building html) in, one binary out, nothing else in the toolchain.

#[path = "../src/lib.rs"]
mod furrow;

use furrow::html::{Page, Shell};
use std::path::Path;

const SHELL: Shell = Shell {
    lang: "en",
    og_url: "",
    head: r#"<style>body{font:18px/1.6 system-ui;max-width:42rem;margin:3rem auto;padding:0 1rem}</style>"#,
};

const CONTENT: &str = "\
# planted with furrow

This page came out of one `rustc` invocation and zero dependencies.

- `furrow::md` rendered this markdown
- `furrow::html` wrapped it in a shell you control
- `furrow::serve` is hosting it right now

## make it yours

Copy `src/` into your project as `mod furrow;`, write your pages as plain
Rust, and emit them with `furrow::out::write_page`.
";

fn main() {
    let dist = Path::new("dist");
    let page = Page {
        title: "planted with furrow",
        description: "a one-file demo site",
        body: &furrow::md::render(CONTENT),
        body_class: "demo",
    };
    furrow::out::write_page(dist, "", &page.render(&SHELL));
    println!("furrow: built dist/");
    if std::env::args().nth(1).as_deref() == Some("serve") {
        furrow::serve::serve(dist, "127.0.0.1:8437", "404 - nothing planted here");
    }
}
