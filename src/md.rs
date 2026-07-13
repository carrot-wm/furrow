// hand-rolled renderer for the markdown subset the docs actually use:
// headings (with anchor slugs), paragraphs, lists, fenced code, inline
// code/bold/links, tables, and blockquote callouts. std only, like the rest.

pub fn render(src: &str) -> String {
    let mut out = String::with_capacity(src.len() * 2);
    let mut lines = src.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_end();

        // fenced code block
        if let Some(rest) = trimmed.strip_prefix("```") {
            let lang = rest.trim();
            out.push_str("<pre class=\"code\"><code");
            if !lang.is_empty() {
                out.push_str(&format!(" data-lang=\"{}\"", esc(lang)));
            }
            out.push('>');
            for code_line in lines.by_ref() {
                if code_line.trim_end().starts_with("```") {
                    break;
                }
                out.push_str(&esc(code_line));
                out.push('\n');
            }
            out.push_str("</code></pre>\n");
            continue;
        }

        // headings
        if let Some((level, text)) = heading(trimmed) {
            let id = slug(text);
            out.push_str(&format!(
                "<h{level} id=\"{id}\">{}<a class=\"anchor\" href=\"#{id}\" aria-label=\"link to this section\">#</a></h{level}>\n",
                inline(text)
            ));
            continue;
        }

        // horizontal rule
        if trimmed == "---" {
            out.push_str("<hr>\n");
            continue;
        }

        // blockquote callout
        if trimmed.starts_with("> ") || trimmed == ">" {
            let mut quote = String::new();
            push_quote_line(&mut quote, trimmed);
            while let Some(next) = lines.peek() {
                let t = next.trim_end();
                if t.starts_with("> ") || t == ">" {
                    push_quote_line(&mut quote, t);
                    lines.next();
                } else {
                    break;
                }
            }
            out.push_str("<blockquote>");
            for para in quote.split("\n\n") {
                if !para.trim().is_empty() {
                    out.push_str(&format!("<p>{}</p>", inline(para.trim())));
                }
            }
            out.push_str("</blockquote>\n");
            continue;
        }

        // table: consecutive lines starting with |
        if trimmed.starts_with('|') {
            let mut rows: Vec<&str> = vec![trimmed];
            while let Some(next) = lines.peek() {
                let t = next.trim_end();
                if t.starts_with('|') {
                    rows.push(t);
                    lines.next();
                } else {
                    break;
                }
            }
            out.push_str(&table(&rows));
            continue;
        }

        // unordered list
        if trimmed.starts_with("- ") {
            out.push_str("<ul>");
            out.push_str(&format!("<li>{}</li>", inline(&trimmed[2..])));
            while let Some(next) = lines.peek() {
                let t = next.trim_end();
                if let Some(item) = t.strip_prefix("- ") {
                    out.push_str(&format!("<li>{}</li>", inline(item)));
                    lines.next();
                } else {
                    break;
                }
            }
            out.push_str("</ul>\n");
            continue;
        }

        // ordered list
        if ordered_item(trimmed).is_some() {
            out.push_str("<ol>");
            out.push_str(&format!("<li>{}</li>", inline(ordered_item(trimmed).unwrap())));
            while let Some(next) = lines.peek() {
                let t = next.trim_end();
                if let Some(item) = ordered_item(t) {
                    out.push_str(&format!("<li>{}</li>", inline(item)));
                    lines.next();
                } else {
                    break;
                }
            }
            out.push_str("</ol>\n");
            continue;
        }

        // blank line: paragraph break
        if trimmed.is_empty() {
            continue;
        }

        // paragraph: accumulate until blank or structural line
        let mut para = String::from(trimmed);
        while let Some(next) = lines.peek() {
            let t = next.trim_end();
            if t.is_empty()
                || t.starts_with('#')
                || t.starts_with("```")
                || t.starts_with("- ")
                || t.starts_with("> ")
                || t.starts_with('|')
                || t == "---"
                || ordered_item(t).is_some()
            {
                break;
            }
            para.push(' ');
            para.push_str(t);
            lines.next();
        }
        out.push_str(&format!("<p>{}</p>\n", inline(&para)));
    }
    out
}

fn push_quote_line(quote: &mut String, line: &str) {
    let content = line.strip_prefix("> ").unwrap_or("");
    if content.is_empty() {
        quote.push_str("\n\n");
    } else {
        if !quote.is_empty() && !quote.ends_with("\n\n") {
            quote.push(' ');
        }
        quote.push_str(content);
    }
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    if (1..=4).contains(&hashes) && line.as_bytes().get(hashes) == Some(&b' ') {
        Some((hashes, line[hashes + 1..].trim()))
    } else {
        None
    }
}

fn ordered_item(line: &str) -> Option<&str> {
    let digits = line.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digits > 0 && line[digits..].starts_with(". ") {
        Some(&line[digits + 2..])
    } else {
        None
    }
}

fn table(rows: &[&str]) -> String {
    let mut out = String::from("<div class=\"table-scroll\"><table>");
    let mut body_started = false;
    for (i, row) in rows.iter().enumerate() {
        let cells: Vec<&str> = row
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim())
            .collect();
        if cells.iter().all(|c| !c.is_empty() && c.bytes().all(|b| b == b'-' || b == b':')) {
            continue;
        }
        let tag = if i == 0 { "th" } else { "td" };
        if i > 0 && !body_started {
            out.push_str("<tbody>");
            body_started = true;
        }
        if i == 0 {
            out.push_str("<thead>");
        }
        out.push_str("<tr>");
        for cell in cells {
            out.push_str(&format!("<{tag}>{}</{tag}>", inline(cell)));
        }
        out.push_str("</tr>");
        if i == 0 {
            out.push_str("</thead>");
        }
    }
    if body_started {
        out.push_str("</tbody>");
    }
    out.push_str("</table></div>\n");
    out
}

fn inline(src: &str) -> String {
    let mut out = String::with_capacity(src.len() + 16);
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            if let Some(end) = src[i + 1..].find('`') {
                out.push_str("<code>");
                out.push_str(&esc(&src[i + 1..i + 1 + end]));
                out.push_str("</code>");
                i += end + 2;
                continue;
            }
        }
        if src[i..].starts_with("**") {
            if let Some(end) = src[i + 2..].find("**") {
                out.push_str("<strong>");
                out.push_str(&inline(&src[i + 2..i + 2 + end]));
                out.push_str("</strong>");
                i += end + 4;
                continue;
            }
        }
        if bytes[i] == b'[' {
            if let Some(close) = src[i..].find("](") {
                if let Some(end) = src[i + close + 2..].find(')') {
                    let text = &src[i + 1..i + close];
                    let url = &src[i + close + 2..i + close + 2 + end];
                    out.push_str(&format!("<a href=\"{}\">{}</a>", esc(url), inline(text)));
                    i += close + 2 + end + 1;
                    continue;
                }
            }
        }
        match bytes[i] {
            b'&' => out.push_str("&amp;"),
            b'<' => out.push_str("&lt;"),
            b'>' => out.push_str("&gt;"),
            _ => out.push(src[i..].chars().next().map(|c| c).unwrap_or('?')),
        }
        i += src[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    }
    out
}

fn esc(s: &str) -> String {
    super::html::esc(s)
}

pub fn slug(text: &str) -> String {
    let mut s = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if (c == ' ' || c == '-' || c == '_') && !s.ends_with('-') {
            s.push('-');
        }
    }
    s.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_get_ids_and_anchors() {
        let html = render("## Window rules\n");
        assert!(html.contains("<h2 id=\"window-rules\">"));
        assert!(html.contains("href=\"#window-rules\""));
    }

    #[test]
    fn inline_code_is_escaped() {
        let html = render("use `<xdg_surface>` here\n");
        assert!(html.contains("<code>&lt;xdg_surface&gt;</code>"));
    }

    #[test]
    fn tables_render_with_header() {
        let html = render("| key | type |\n|---|---|\n| `size` | int |\n");
        assert!(html.contains("<th>key</th>"));
        assert!(html.contains("<td><code>size</code></td>"));
    }

    #[test]
    fn fences_preserve_content_verbatim() {
        let html = render("```kdl\nbind \"Super\" \"Return\" \"exec\" \"foot\"\n```\n");
        assert!(html.contains("data-lang=\"kdl\""));
        assert!(html.contains("bind &quot;Super&quot;"));
    }

    #[test]
    fn bold_and_links_nest_in_paragraphs() {
        let html = render("see **the [docs](/docs/) page** now\n");
        assert!(html.contains("<strong>the <a href=\"/docs/\">docs</a> page</strong>"));
    }
}
