// a dev server in a page of std. one thread, one connection at a time;
// it's a dev server.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;

pub fn serve(dist: &Path, addr: &str, not_found: &str) {
    let listener = TcpListener::bind(addr).expect("bind");
    println!("furrow: serving {} on http://{addr}", dist.display());
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut buf = [0u8; 2048];
        let Ok(n) = stream.read(&mut buf) else { continue };
        let req = String::from_utf8_lossy(&buf[..n]);
        let path = req.split_whitespace().nth(1).unwrap_or("/");
        let path = path.split(['?', '#']).next().unwrap_or("/");
        let rel = match path {
            "/" => "index.html".to_string(),
            p => {
                let p = p.trim_start_matches('/');
                // no dotfiles, no traversal
                if p.contains("..") || p.starts_with('.') {
                    respond(&mut stream, 403, "text/plain", b"forbidden");
                    continue;
                }
                p.to_string()
            }
        };
        let file = dist.join(&rel);
        let file = if file.is_dir() { file.join("index.html") } else { file };
        // mime from the file actually served, not the request path: /about/
        // resolves to about/index.html and must ship as html, not a download
        let ctype = mime(&file.to_string_lossy());
        match fs::read(&file) {
            Ok(body) => respond(&mut stream, 200, ctype, &body),
            Err(_) => respond(&mut stream, 404, "text/plain", not_found.as_bytes()),
        }
    }
}

fn mime(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "woff2" => "font/woff2",
        "png" => "image/png",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn respond(stream: &mut std::net::TcpStream, code: u16, ctype: &str, body: &[u8]) {
    let status = match code {
        200 => "200 OK",
        403 => "403 Forbidden",
        _ => "404 Not Found",
    };
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}
