//! WCAG 2.2 AA contrast of the theme tokens in both themes (SC-005, research R11).

const TOKENS: &str = include_str!("../styles/tokens.css");

/// Text tokens and the backgrounds they are drawn on, with the minimum ratio.
const TEXT_PAIRS: &[(&str, &str, f64)] = &[
    ("--text", "--bg", 4.5),
    ("--text", "--surface", 4.5),
    ("--text-body", "--bg", 4.5),
    ("--text-body", "--surface", 4.5),
    ("--text-muted", "--bg", 4.5),
    ("--text-muted", "--surface", 4.5),
    ("--accent", "--bg", 4.5),
    ("--accent", "--surface", 4.5),
    ("--reasoning", "--surface", 4.5),
    ("--ok", "--surface", 4.5),
    ("--warn", "--surface", 4.5),
    ("--error", "--surface", 4.5),
    ("--error", "--bg", 4.5),
    ("--info", "--surface", 4.5),
    ("--on-primary", "--primary", 4.5),
    // Focus indicators and the active navigation border: 3:1 against what they sit on.
    ("--focus", "--bg", 3.0),
    ("--focus", "--surface", 3.0),
    ("--primary", "--surface", 3.0),
];

fn theme_block(theme: &str) -> &'static str {
    let marker = format!("[data-theme=\"{theme}\"]");
    let start = TOKENS
        .find(&marker)
        .unwrap_or_else(|| panic!("no {theme} theme"));
    let open = start + TOKENS[start..].find('{').expect("block opens");
    let close = open + TOKENS[open..].find('}').expect("block closes");
    &TOKENS[open + 1..close]
}

fn token(block: &str, name: &str) -> [u8; 3] {
    let line = block
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with(&format!("{name}:")))
        .unwrap_or_else(|| panic!("token {name} missing"));
    let hex = line
        .split('#')
        .nth(1)
        .and_then(|rest| rest.get(..6))
        .unwrap_or_else(|| panic!("token {name} is not #RRGGBB"));
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex digit");
    [byte(0), byte(2), byte(4)]
}

fn luminance(rgb: [u8; 3]) -> f64 {
    let channel = |value: u8| {
        let c = f64::from(value) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(rgb[0]) + 0.7152 * channel(rgb[1]) + 0.0722 * channel(rgb[2])
}

fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

#[test]
fn the_formula_matches_known_values() {
    assert!((contrast([0, 0, 0], [255, 255, 255]) - 21.0).abs() < 0.01);
    // AnyContext's muted text on its background: the reason Aurora lightens it.
    assert!(contrast([0x64, 0x74, 0x8B], [0x0B, 0x0F, 0x19]) < 4.5);
}

#[test]
fn every_pair_meets_wcag_aa_in_both_themes() {
    let mut failures = Vec::new();
    for theme in ["dark", "light"] {
        let block = theme_block(theme);
        for (fg, bg, minimum) in TEXT_PAIRS {
            let ratio = contrast(token(block, fg), token(block, bg));
            if ratio < *minimum {
                failures.push(format!(
                    "{theme}: {fg} on {bg} is {ratio:.2}:1, needs {minimum}:1"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
