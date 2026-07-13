// the smallest html layer that stays honest: escape text, wrap the page shell.
// structure lives in your page modules as plain rust: no templates, no macros.

/// escape untrusted-ish text for element content and attribute values.
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// the site-wide parts of the shell, owned by your site, written once.
/// `head` is raw html appended before </head>: icons, font preloads,
/// stylesheets, whatever the site self-hosts.
pub struct Shell<'a> {
    pub lang: &'a str,
    /// og:url for the site; empty skips the tag.
    pub og_url: &'a str,
    pub head: &'a str,
}

pub struct Page<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub body: &'a str,
    pub body_class: &'a str,
}

impl Page<'_> {
    pub fn render(&self, shell: &Shell) -> String {
        let og_url = if shell.og_url.is_empty() {
            String::new()
        } else {
            format!("<meta property=\"og:url\" content=\"{}\">\n", esc(shell.og_url))
        };
        format!(
            r#"<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>{title}</title>
<meta name="description" content="{desc}">
<meta property="og:title" content="{title}">
<meta property="og:description" content="{desc}">
<meta property="og:type" content="website">
{og_url}{head}
</head>
<body class="{cls}">
{body}
</body>
</html>
"#,
            lang = shell.lang,
            title = esc(self.title),
            desc = esc(self.description),
            og_url = og_url,
            head = shell.head,
            body = self.body,
            cls = self.body_class,
        )
    }
}
