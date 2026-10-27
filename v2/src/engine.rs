//! Literal port of own V1 source; intentionally not a general CSS parser.
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::LazyLock;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Row {
    pub title: String,
    pub subtitle: String,
    pub accessory: String,
    pub copy: String,
    pub swatch: Option<String>,
}
pub fn num(n: f64) -> String {
    if n.is_nan() {
        return "NaN".into();
    }
    if n == f64::INFINITY {
        return "Infinity".into();
    }
    if n == f64::NEG_INFINITY {
        return "-Infinity".into();
    }
    if n == 0.0 {
        return "0".into();
    }
    ryu_js::Buffer::new().format(n).into()
}
pub fn round(n: f64) -> f64 {
    if !n.is_finite() || n == 0.0 {
        return n;
    }
    let f = n.floor();
    let r = if n - f < 0.5 { f } else { f + 1.0 };
    if r == 0.0 && n < 0.0 {
        -0.0
    } else {
        r
    }
}
fn js_space(c: char) -> bool {
    matches!(c, '\u{0009}'..='\u{000d}' | ' ' | '\u{00a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}
fn number(s: &str) -> f64 {
    let s = s.trim_matches(js_space);
    if s.is_empty() {
        return 0.0;
    }
    if matches!(s, "Infinity" | "+Infinity") {
        return f64::INFINITY;
    }
    if s == "-Infinity" {
        return f64::NEG_INFINITY;
    }
    if let Some((radix, digits)) = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .map(|s| (16, s))
        .or_else(|| {
            s.strip_prefix("0b")
                .or_else(|| s.strip_prefix("0B"))
                .map(|s| (2, s))
        })
        .or_else(|| {
            s.strip_prefix("0o")
                .or_else(|| s.strip_prefix("0O"))
                .map(|s| (8, s))
        })
    {
        if digits.is_empty() {
            return f64::NAN;
        }
        return digits
            .chars()
            .try_fold(0.0, |a, c| {
                c.to_digit(radix)
                    .map(|d| a * f64::from(radix) + f64::from(d))
            })
            .unwrap_or(f64::NAN);
    }
    static DEC: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?$").unwrap()
    });
    if !DEC.is_match(s) {
        return f64::NAN;
    }
    s.parse().unwrap_or(f64::NAN)
}
fn primitive_string(v: &Value, array: bool) -> String {
    match v {
        Value::Null => {
            if array {
                String::new()
            } else {
                "null".into()
            }
        }
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => num(n.as_f64().unwrap_or(f64::NAN)),
        Value::Array(a) => a
            .iter()
            .map(|x| primitive_string(x, true))
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".into(),
    }
}
pub fn base(preference: Option<&Value>) -> f64 {
    let n = match preference {
        None => 16.0,
        Some(Value::String(s)) if s.is_empty() => 16.0,
        Some(Value::Null) => 0.0,
        Some(Value::Bool(b)) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Some(v) => number(&primitive_string(v, false)),
    };
    if n.is_nan() {
        16.0
    } else {
        n
    }
}
fn rx(s: &str) -> Regex {
    // ECMAScript whitespace and ASCII digits, not Rust's Unicode digit class.
    Regex::new(&s.replace(r"\d","[0-9]").replace(r"\s",r"[\x09-\x0d\x20\u{a0}\u{1680}\u{2000}-\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}]")).unwrap()
}
static UNITS: LazyLock<[Regex; 3]> = LazyLock::new(|| {
    [
        rx(
            r"(?i)(\d+|^[\x00-\u{ffff}&&[^\n\r\x{2028}\x{2029}]]\d+|^,\d+|^\d+,\d+|^\d+[\x00-\u{ffff}&&[^\n\r\x{2028}\x{2029}]]\d+)(\srem|rem)",
        ),
        rx(
            r"(\d+|^[\x00-\u{ffff}&&[^\n\r\x{2028}\x{2029}]]\d+|^,\d+|^\d+,\d+|^\d+[\x00-\u{ffff}&&[^\n\r\x{2028}\x{2029}]]\d+)(\spx|px)",
        ),
        rx(
            r"(?i)(\d+|^[\x00-\u{ffff}&&[^\n\r\x{2028}\x{2029}]]\d+|^,\d+|^\d+,\d+|^\d+[\x00-\u{ffff}&&[^\n\r\x{2028}\x{2029}]]\d+)(\spt|pt)",
        ),
    ]
});
static HEX: LazyLock<Regex> =
    LazyLock::new(|| rx(r"(?i)^#([a-f0-9]{6}|[a-f0-9]{3})([a-f0-9]{2})?$"));
static RGB: LazyLock<Regex> = LazyLock::new(|| {
    rx(
        r"(?i)^rgba?\(\s*(\d{1,3}%?)\s*(?:,\s*|\s+)(\d{1,3}%?)\s*(?:,\s*|\s+)(\d{1,3}%?)\s*(?:[,/]\s*((?:\d+\.?\d*|\.?\d+)%?))?\s*\)$",
    )
});
static HSL: LazyLock<Regex> = LazyLock::new(|| {
    rx(
        r"(?i)^hsla?\(\s*(\d{1,3})\s*(?:,\s*|\s+)(\d{1,3})%?\s*(?:,\s*|\s+)(\d{1,3})%?\s*(?:[,/]\s*((?:\d+\.?\d*|\.?\d+)%?))?\s*\)$",
    )
});
static OKLCH: LazyLock<Regex> =
    LazyLock::new(|| rx(r"(?i)^oklch\(\s*([\d.]+)(%)?\s+([\d.]+)\s+([\d.]+)\s*\)$"));
fn parse_float(s: &str) -> f64 {
    // Used only for the source's positive [digits/dots] captures.
    let mut dot = false;
    let prefix: String = s
        .chars()
        .take_while(|c| {
            if *c == '.' {
                if dot {
                    false
                } else {
                    dot = true;
                    true
                }
            } else {
                c.is_ascii_digit()
            }
        })
        .collect();
    prefix.parse().unwrap_or(f64::NAN)
}
fn alpha(s: &str) -> f64 {
    if s.contains('%') {
        parse_float(s) / 100.0
    } else {
        number(s)
    }
}
fn channel(s: &str) -> f64 {
    if s.ends_with('%') {
        round(parse_float(s) / 100.0 * 255.0)
    } else {
        number(s)
    }
}
fn hex_rgb(s: &str) -> [f64; 3] {
    let s = s.trim_start_matches('#');
    if s.len() == 3 {
        let b = s.as_bytes();
        std::array::from_fn(|i| f64::from((b[i] as char).to_digit(16).unwrap() * 17))
    } else if s.len() == 6 {
        std::array::from_fn(|i| f64::from(u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap()))
    } else {
        [0.0; 3]
    }
}
fn hex_num(n: f64) -> String {
    if !n.is_finite() {
        return num(n);
    }
    if n == 0.0 {
        return "00".into();
    }
    // All product callers pass rounded integer channels. Huge alpha can exceed u128.
    let neg = n < 0.0;
    let mut n = n.abs();
    let mut chars = Vec::new();
    while n >= 1.0 {
        let d = (n % 16.0) as u32;
        chars.push(char::from_digit(d.min(15), 16).unwrap());
        n = (n / 16.0).floor();
    }
    if neg {
        chars.push('-');
    }
    let mut out: String = chars.into_iter().rev().collect();
    if out.len() < 2 {
        out.insert(0, '0');
    }
    out
}
fn rgb_hex(v: [f64; 3]) -> String {
    format!("#{}{}{}", hex_num(v[0]), hex_num(v[1]), hex_num(v[2]))
}
fn rgb_hsl(v: [f64; 3]) -> [f64; 3] {
    let [r, g, b] = v.map(|x| x / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let mut h = 0.0;
    let mut s = 0.0;
    let l = (max + min) / 2.0;
    if max != min {
        let d = max - min;
        s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };
        if max == r {
            h = (g - b) / d + if g < b { 6.0 } else { 0.0 };
        } else if max == g {
            h = (b - r) / d + 2.0;
        } else if max == b {
            h = (r - g) / d + 4.0;
        }
        h /= 6.0;
    }
    if v.iter().any(|n| n.is_nan()) {
        return [0.0, f64::NAN, f64::NAN];
    }
    [
        round((h * 360.0) % 360.0),
        round(s * 100.0),
        round(l * 100.0),
    ]
}
fn hsl_rgb([h, s, l]: [f64; 3]) -> [f64; 3] {
    let s = s / 100.0;
    let l = l / 100.0;
    let a = s * l.min(1.0 - l);
    [0.0, 8.0, 4.0].map(|n| {
        let k = (n + h / 30.0) % 12.0;
        round(255.0 * (l - a * (-1.0_f64).max((k - 3.0).min((9.0 - k).min(1.0)))))
    })
}
// Preserve the source operation order for floating point parity.
#[allow(clippy::manual_clamp)]
fn hsl_hex([h, s, l]: [f64; 3]) -> String {
    let l = l / 100.0;
    let a = (s * l.min(1.0 - l)) / 100.0;
    rgb_hex([0.0, 8.0, 4.0].map(|n| {
        let k = (n + h / 30.0) % 12.0;
        round(255.0 * (l - a * (k - 3.0).min(9.0 - k).min(1.0).max(-1.0)))
    }))
}
fn linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
fn rgb_oklch(v: [f64; 3]) -> [f64; 3] {
    let [r, g, b] = v.map(|c| linear(c / 255.0));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    let ll = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
    let a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
    let bb = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
    let c = (a * a + bb * bb).sqrt();
    let mut h = bb.atan2(a) * (180.0 / std::f64::consts::PI);
    if h < 0.0 {
        h += 360.0;
    }
    [
        round(ll * 1000.0) / 1000.0,
        round(c * 1000.0) / 1000.0,
        round(h * 10.0) / 10.0,
    ]
}
fn oklch_rgb([l, c, h]: [f64; 3]) -> [f64; 3] {
    let rad = h * std::f64::consts::PI / 180.0;
    let a = c * rad.cos();
    let b = c * rad.sin();
    let ll = l + 0.3963377774 * a + 0.2158037573 * b;
    let mm = l - 0.1055613458 * a - 0.0638541728 * b;
    let ss = l - 0.0894841775 * a - 1.291485548 * b;
    let l = ll * ll * ll;
    let m = mm * mm * mm;
    let s = ss * ss * ss;
    [
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
    ]
    .map(|c| {
        let s = srgb(c);
        if s.is_nan() {
            s
        } else {
            round(s.clamp(0.0, 1.0) * 255.0)
        }
    })
}
static COLORS: LazyLock<Vec<(String, String)>> =
    LazyLock::new(|| serde_json::from_str(include_str!("tailwind.json")).unwrap());
static SPACINGS: LazyLock<Vec<(String, f64, f64)>> =
    LazyLock::new(|| serde_json::from_str(include_str!("spacings.json")).unwrap());
fn closest(rgb: [f64; 3]) -> (String, String) {
    let mut best = (String::new(), String::new());
    let mut dist = 9007199254740991.0;
    for (name, hex) in COLORS.iter() {
        let c = hex_rgb(hex);
        let d =
            ((rgb[0] - c[0]).powi(2) + (rgb[1] - c[1]).powi(2) + (rgb[2] - c[2]).powi(2)).sqrt();
        if d < dist {
            dist = d;
            best = (name.clone(), hex.clone());
        }
    }
    best
}
fn spacing(n: f64, rem: bool) -> String {
    SPACINGS
        .iter()
        .find(|(_, r, p)| if rem { *r == n } else { *p == n })
        .map(|(s, _, _)| s.clone())
        .unwrap_or_else(|| format!("[{}{}]", num(n), if rem { "rem" } else { "px" }))
}
fn joined(v: &[f64], sep: &str) -> String {
    v.iter().map(|n| num(*n)).collect::<Vec<_>>().join(sep)
}
fn hsl_text(v: [f64; 3]) -> String {
    format!("hsl({}, {}%, {}%)", num(v[0]), num(v[1]), num(v[2]))
}

pub fn convert(input: &str, preference: Option<&Value>) -> Vec<Row> {
    if input.is_empty() {
        return vec![];
    }
    let base = base(preference);
    let (mut rem, mut px, mut pt) = (None, None, None);
    let (mut hex, mut hexa) = (None, None);
    let (mut rgb, mut rgba, mut hsl, mut hsla, mut oklch) = (None, None, None, None, None);
    let (mut tw, mut color) = (None, None);
    for (i, rx) in UNITS.iter().enumerate() {
        if let Some(c) = rx.captures(input) {
            let n = number(&c[1]);
            match i {
                0 => {
                    px = Some(n * base);
                    pt = Some(n * 12.0);
                    tw = Some(spacing(n, true));
                }
                1 => {
                    rem = Some(n / base);
                    pt = Some(n * 0.75);
                    tw = Some(spacing(n, false));
                }
                _ => {
                    rem = Some(n / 12.0);
                    px = Some(n / 0.75);
                }
            }
        }
    }
    if let Some(c) = HEX.captures(input) {
        let v = hex_rgb(&c[1]);
        rgb = Some(v);
        hsl = Some(rgb_hsl(v));
        oklch = Some(rgb_oklch(v));
        color = Some(closest(v));
        if c.get(2).is_some() {
            hex = Some(rgb_hex(v));
            hexa = Some(input.into());
            let a = if input.len() == 9 {
                let v = hex_rgb(&input[1..7]);
                [
                    v[0],
                    v[1],
                    v[2],
                    f64::from(u8::from_str_radix(&input[7..9], 16).unwrap()) / 255.0,
                ]
            } else {
                [0.0; 4]
            };
            rgba = Some(a);
            let h = rgb_hsl([a[0], a[1], a[2]]);
            hsla = Some([h[0], h[1], h[2], a[3]]);
        }
    }
    for (is_hsl, rx) in [(false, &*RGB), (true, &*HSL)] {
        if let Some(c) = rx.captures(input) {
            let v = if is_hsl {
                [number(&c[1]), number(&c[2]), number(&c[3])]
            } else {
                [channel(&c[1]), channel(&c[2]), channel(&c[3])]
            };
            let r = if is_hsl { hsl_rgb(v) } else { v };
            let h = if is_hsl { v } else { rgb_hsl(r) };
            let hx = if is_hsl { hsl_hex(v) } else { rgb_hex(r) };
            hex = Some(hx.clone());
            rgb = Some(r);
            hsl = Some(h);
            oklch = Some(rgb_oklch(r));
            color = Some(closest(r));
            if let Some(a) = c.get(4) {
                let a = alpha(a.as_str());
                hexa = Some(format!("{}{}", hx, hex_num(round(a * 255.0))));
                rgba = Some([r[0], r[1], r[2], a]);
                hsla = Some([h[0], h[1], h[2], a]);
            }
        }
    }
    if let Some(c) = OKLCH.captures(input) {
        let l = parse_float(&c[1]) / if c.get(2).is_some() { 100.0 } else { 1.0 };
        let v = [l, parse_float(&c[3]), parse_float(&c[4])];
        let r = oklch_rgb(v);
        oklch = Some(v);
        hex = Some(rgb_hex(r));
        rgb = Some(r);
        hsl = Some(rgb_hsl(r));
        color = Some(closest(r));
    }
    let mut rows = Vec::new();
    let mut add = |title: String,
                   subtitle: String,
                   accessory: &str,
                   copy: Option<String>,
                   swatch: Option<String>| {
        rows.push(Row {
            copy: copy.unwrap_or_else(|| title.clone()),
            title,
            subtitle,
            accessory: accessory.into(),
            swatch,
        });
    };
    for (n, u) in [(rem, "rem"), (px, "px"), (pt, "pt")] {
        if let Some(n) = n {
            if n != 0.0 && !n.is_nan() {
                add(
                    format!("{}{u}", num(n)),
                    String::new(),
                    &format!("to {u}"),
                    None,
                    None,
                );
            }
        }
    }
    if let Some(h) = &hex {
        add(h.clone(), String::new(), "to hex", None, Some(h.clone()));
    }
    if let Some(h) = hexa {
        add(h.clone(), String::new(), "to hexa", None, Some(h));
    }
    if let Some(v) = rgb {
        let s = format!("rgb({})", joined(&v, ", "));
        add(s.clone(), String::new(), "to rgb", None, Some(s));
    }
    if let Some(v) = rgba {
        let s = format!("rgba({})", joined(&v, ", "));
        add(s.clone(), String::new(), "to rgba", None, Some(s));
    }
    if let Some(v) = hsl {
        let s = hsl_text(v);
        add(s.clone(), String::new(), "to hsl", None, Some(s));
    }
    if let Some(v) = hsla {
        let s = format!(
            "hsla({}, {}%, {}%, {})",
            num(v[0]),
            num(v[1]),
            num(v[2]),
            num(v[3])
        );
        add(s.clone(), String::new(), "to hsla", None, Some(s));
    }
    if let Some(v) = oklch {
        let s = format!("oklch({})", joined(&v, " "));
        add(
            s,
            String::new(),
            "to oklch",
            None,
            Some(hsl_text(hsl.unwrap_or([0.0; 3]))),
        );
    }
    if let Some((name, h)) = color {
        let close = input != h && hex.as_deref() != Some(&h);
        let copy = if input != h { h.clone() } else { name.clone() };
        add(
            if close { h.clone() } else { name.clone() },
            if close { name } else { String::new() },
            if close {
                "closest Tailwind color"
            } else {
                "Tailwind color"
            },
            Some(copy),
            Some(h),
        );
    }
    if let Some(s) = tw {
        let a = if s.contains(']') {
            "to Tailwind Arbitrary Value"
        } else {
            "to Tailwind Spacing"
        };
        add(s, String::new(), a, None, None);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn number_semantics() {
        assert_eq!(num(-0.0), "0");
        assert!(round(-0.5).is_sign_negative());
        assert_eq!(num(1e21), "1e+21");
        assert_eq!(base(Some(&serde_json::json!(" "))), 0.0);
    }
    #[test]
    fn source_oddities() {
        assert!(convert("0px", None).iter().all(|r| r.title == "0"));
        assert!(convert("10PX", None).is_empty());
        assert_eq!(convert("-12px", None)[0].title, "-0.75rem");
        assert_eq!(spacing(1.0, true), "4");
    }
}
