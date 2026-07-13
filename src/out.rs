// emitting the site: the two filesystem moves every build makes.

use std::fs;
use std::path::Path;

/// write one page under its pretty route: "" lands at index.html,
/// "about" at about/index.html, "docs/tiling" at docs/tiling/index.html.
pub fn write_page(dist: &Path, route: &str, html: &str) {
    let dir = if route.is_empty() { dist.to_path_buf() } else { dist.join(route) };
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    let file = dir.join("index.html");
    fs::write(&file, html).unwrap_or_else(|e| panic!("write {}: {e}", file.display()));
}

/// copy a directory tree into the dist root, as-is. static assets, fonts,
/// favicons; anything the pages reference by absolute path.
pub fn copy_tree(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap_or_else(|e| panic!("read {}: {e}", from.display())) {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            fs::create_dir_all(&dest).unwrap();
            copy_tree(&entry.path(), &dest);
        } else {
            fs::copy(entry.path(), dest).unwrap();
        }
    }
}
