// build-time github data: stars and descriptions, fetched with a curl
// subprocess (no crates, no tokens) and hand-rolled field extraction.
// offline or rate-limited builds fall back to the last numbers you saw.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RepoData {
    pub stars: u64,
    pub description: String,
    pub live: bool,
}

/// fetch one repo's stars and description; `ua` is the User-Agent your site
/// identifies as. falls back to the supplied numbers when the network says no.
pub fn fetch(slug: &str, fallback_stars: u64, fallback_desc: &str, ua: &str) -> RepoData {
    let url = format!("https://api.github.com/repos/{slug}");
    let agent = format!("User-Agent: {ua}");
    let out = Command::new("curl")
        .args(["-sf", "--max-time", "5", "-H", &agent, &url])
        .output();
    if let Ok(out) = out {
        if out.status.success() {
            let body = String::from_utf8_lossy(&out.stdout);
            if let Some(stars) = json_u64(&body, "stargazers_count") {
                let description = json_str(&body, "description")
                    .unwrap_or_else(|| fallback_desc.to_string());
                return RepoData { stars, description, live: true };
            }
        }
    }
    RepoData {
        stars: fallback_stars,
        description: fallback_desc.to_string(),
        live: false,
    }
}

pub struct Asset {
    pub name: String,
    pub live: bool,
}

/// the newest release asset whose filename ends with `suffix` (say,
/// "_amd64.deb"), read from /releases/latest; `fallback` is the last
/// name you saw, for offline or rate-limited builds.
pub fn latest_asset(slug: &str, suffix: &str, fallback: &str, ua: &str) -> Asset {
    let url = format!("https://api.github.com/repos/{slug}/releases/latest");
    let agent = format!("User-Agent: {ua}");
    let out = Command::new("curl")
        .args(["-sf", "--max-time", "5", "-H", &agent, &url])
        .output();
    if let Ok(out) = out {
        if out.status.success() {
            let body = String::from_utf8_lossy(&out.stdout);
            if let Some(name) = asset_name(&body, suffix) {
                return Asset { name, live: true };
            }
        }
    }
    Asset { name: fallback.to_string(), live: false }
}

/// scan every "browser_download_url" for a basename ending in `suffix`;
/// download urls only exist on assets, so the release's own "name"
/// field can never shadow them.
fn asset_name(body: &str, suffix: &str) -> Option<String> {
    let needle = "\"browser_download_url\":\"";
    let mut rest = body;
    while let Some(at) = rest.find(needle) {
        rest = &rest[at + needle.len()..];
        let end = rest.find('"')?;
        let base = rest[..end].rsplit('/').next().unwrap_or("");
        if base.ends_with(suffix) {
            return Some(base.to_string());
        }
        rest = &rest[end..];
    }
    None
}

/// scan for `"key":<number>` at top level-ish. good enough for github's flat
/// repo object; not a json parser and doesn't pretend to be.
fn json_u64(body: &str, key: &str) -> Option<u64> {
    let needle = format!("\"{key}\":");
    let at = body.find(&needle)? + needle.len();
    let rest = body[at..].trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit())?;
    rest[..end].parse().ok()
}

/// scan for `"key":"..."`, un-escaping the couple of sequences github actually
/// emits in descriptions.
fn json_str(body: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let at = body.find(&needle)? + needle.len();
    let mut out = String::new();
    let mut chars = body[at..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push(' '),
                't' => out.push(' '),
                'u' => {
                    // \uXXXX: take the hex, emit the char, shrug at surrogates
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        out.push(ch);
                    }
                }
                other => out.push(other),
            },
            _ => out.push(c),
        }
    }
    None
}

/// today as YYYY-MM-DD from the system clock; civil-from-days per the
/// classic Hinnant algorithm. build metadata, so local-enough is fine.
pub fn build_date() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86400) as i64;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}
