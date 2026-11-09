//! Save slots and settings in the LÖVE port's folder and formats, readable and writable by both.
//!
//! - `save_<slot+1>.json` / `save_<slot+1>_meta.json`: JSON text followed by one NUL byte
//!   (`save_write_text` writes a GML string into a buffer), written to `<path>.tmp` then renamed.
//!   Reading stops at the first NUL like `buffer_read(buffer_string)`.
//! - JSON text is produced like `gm.json` (`game/gm/json.lua`): object keys sorted, integral
//!   numbers below 1e15 as integers, other numbers `%.17g`, so a LÖVE save written back by Rust
//!   is byte-identical.
//! - `settings.ini`: `display_math.settings_to_ini` / `ini_to_settings`.

use crate::state::{DisplaySettings, GameState, SlotMeta};
use serde::Serialize;
use serde_json::Value;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// `~/Library/Application Support/LOVE/Mycelarium` (LÖVE identity "Mycelarium" on macOS).
pub fn save_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join("Library/Application Support/LOVE/Mycelarium")
}

pub fn save_path(dir: &Path, slot: usize) -> PathBuf {
    dir.join(format!("save_{}.json", slot + 1))
}

pub fn meta_path(dir: &Path, slot: usize) -> PathBuf {
    dir.join(format!("save_{}_meta.json", slot + 1))
}

#[derive(Debug)]
pub enum SaveError {
    /// no save file for the slot
    Missing,
    Io(io::Error),
    /// not JSON (truncated, garbage) or not the save shape
    Json(serde_json::Error),
    /// `version` other than 1 (missing counts as 0), as load_game rejects it
    Version(i64),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Missing => write!(f, "no save file"),
            SaveError::Io(e) => write!(f, "read error: {e}"),
            SaveError::Json(e) => write!(f, "bad save data: {e}"),
            SaveError::Version(v) => write!(f, "unsupported save version {v}"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<io::Error> for SaveError {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::NotFound {
            SaveError::Missing
        } else {
            SaveError::Io(e)
        }
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(e: serde_json::Error) -> Self {
        SaveError::Json(e)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SlotStatus {
    Empty,
    Ok(SlotMeta),
    Corrupt(String),
}

/// File text up to the first NUL (`save_read_text`).
fn read_text(path: &Path) -> Result<Vec<u8>, SaveError> {
    let mut bytes = std::fs::read(path)?;
    if let Some(end) = bytes.iter().position(|&b| b == 0) {
        bytes.truncate(end);
    }
    Ok(bytes)
}

/// `save_write_text`: text + NUL into `<path>.tmp`, then replace `path`.
fn write_text(path: &Path, text: &str) -> io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let mut bytes = Vec::with_capacity(text.len() + 1);
    bytes.extend_from_slice(text.as_bytes());
    bytes.push(0);
    let res = std::fs::write(&tmp, &bytes).and_then(|_| std::fs::rename(&tmp, path));
    if res.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    res
}

pub fn read_slot(dir: &Path, slot: usize) -> Result<GameState, SaveError> {
    let text = read_text(&save_path(dir, slot))?;
    let value: Value = serde_json::from_slice(&text)?;
    let version = value.get("version").and_then(Value::as_f64).unwrap_or(0.0);
    if version != 1.0 {
        return Err(SaveError::Version(version as i64));
    }
    let mut state: GameState = serde_json::from_value(value)?;
    state.apply_load_defaults();
    Ok(state)
}

/// Write the save then its meta, like `save_game`.
pub fn write_slot(dir: &Path, slot: usize, state: &GameState, meta: &SlotMeta) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    write_text(&save_path(dir, slot), &to_love_json(state)?)?;
    write_text(&meta_path(dir, slot), &to_love_json(meta)?)
}

pub fn read_meta(dir: &Path, slot: usize) -> Result<SlotMeta, SaveError> {
    let text = read_text(&meta_path(dir, slot))?;
    Ok(serde_json::from_slice(&text)?)
}

/// Slot list for the menu. Like `save_slot_meta`, a slot without a meta file is empty; a broken
/// meta or a save that does not load is reported as corrupt instead of failing.
pub fn list_slots(dir: &Path, count: usize) -> Vec<SlotStatus> {
    (0..count)
        .map(|slot| match read_meta(dir, slot) {
            Err(SaveError::Missing) => SlotStatus::Empty,
            Err(e) => SlotStatus::Corrupt(format!("meta: {e}")),
            Ok(meta) => match read_slot(dir, slot) {
                Ok(_) => SlotStatus::Ok(meta),
                Err(e) => SlotStatus::Corrupt(e.to_string()),
            },
        })
        .collect()
}

/// `ini_to_settings`: defaults for a missing file, section or bad value.
pub fn read_settings(dir: &Path) -> DisplaySettings {
    let text = std::fs::read(dir.join("settings.ini")).unwrap_or_default();
    parse_settings(&String::from_utf8_lossy(&text))
}

pub fn parse_settings(text: &str) -> DisplaySettings {
    let mut s = DisplaySettings::default();
    let mut in_display = false;
    for line in text.split(['\r', '\n']).filter(|l| !l.is_empty()) {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') && t.len() >= 2 {
            in_display = &t[1..t.len() - 1] == "display";
            continue;
        }
        if !in_display {
            continue;
        }
        let Some((k, v)) = t.split_once('=') else {
            continue;
        };
        let k = k.trim();
        if k.is_empty() || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let v = v.trim();
        let v = v.strip_prefix('"').unwrap_or(v);
        let v = v.strip_suffix('"').unwrap_or(v);
        if v.contains('"') {
            continue;
        }
        let Ok(n) = v.trim().parse::<f64>() else {
            continue;
        };
        if !n.is_finite() {
            continue;
        }
        match k {
            "fullscreen" => s.fullscreen = n != 0.0,
            "window_w" | "window_h" if n > 0.0 => {
                let px = n.floor().min(u32::MAX as f64) as u32;
                if k == "window_w" {
                    s.window_w = px
                } else {
                    s.window_h = px
                }
            }
            _ => {}
        }
    }
    s
}

/// `settings_to_ini`, byte for byte (CRLF, values as `%f`).
pub fn settings_to_ini(s: &DisplaySettings) -> String {
    format!(
        "[display]\r\nfullscreen=\"{:.6}\"\r\nwindow_w=\"{:.6}\"\r\nwindow_h=\"{:.6}\"\r\n",
        if s.fullscreen { 1.0 } else { 0.0 },
        s.window_w as f64,
        s.window_h as f64
    )
}

/// `love.filesystem.write` (no NUL, not through a temp file in LÖVE; a temp file here is harmless).
pub fn write_settings(dir: &Path, s: &DisplaySettings) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join("settings.ini");
    let tmp = dir.join("settings.ini.tmp");
    std::fs::write(&tmp, settings_to_ini(s))?;
    std::fs::rename(&tmp, &path)
}

// ---------------------------------------------------------------------------------------------
// JSON text in gm.json's format

pub fn to_love_json<T: Serialize>(v: &T) -> io::Result<String> {
    let value =
        serde_json::to_value(v).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut out = String::new();
    encode(&value, &mut out);
    Ok(out)
}

fn encode(v: &Value, out: &mut String) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                encode_number(i as f64, Some(i), out)
            } else if let Some(u) = n.as_u64() {
                encode_number(u as f64, None, out)
            } else {
                encode_number(n.as_f64().unwrap_or(0.0), None, out)
            }
        }
        Value::String(s) => encode_string(s, out),
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                encode(x, out);
            }
            out.push(']');
        }
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                encode_string(k, out);
                out.push(':');
                encode(&m[k], out);
            }
            out.push('}');
        }
    }
}

/// gm.json encode_number: NaN/inf -> 0, integral below 1e15 -> `%d`, else `%.17g`.
fn encode_number(n: f64, exact: Option<i64>, out: &mut String) {
    if !n.is_finite() {
        out.push('0');
    } else if let Some(i) = exact.filter(|i| i.unsigned_abs() < 1_000_000_000_000_000) {
        out.push_str(&i.to_string());
    } else if n == n.floor() && n.abs() < 1e15 {
        out.push_str(&(n as i64).to_string());
    } else {
        out.push_str(&format_g17(n));
    }
}

/// Exact decimal digits of |x| (`n` significant, truncated by Rust's correct rounding only
/// beyond them) and the decimal exponent.
fn sci_digits(x: f64, n: usize) -> (String, i32) {
    let sci = format!("{:.*e}", n - 1, x.abs());
    let (mant, exp) = sci.split_once('e').unwrap();
    (
        mant.chars().filter(|c| c.is_ascii_digit()).collect(),
        exp.parse().unwrap(),
    )
}

/// The 17 significant digits LuaJIT's `string.format("%.17g")` prints: the exact value rounded
/// with ties away from zero (Rust's own formatting rounds ties to even, which differs on
/// doubles whose exact expansion ends in a 5 at the 18th digit, e.g. 879.090240478515625).
fn digits17(x: f64) -> (String, i32) {
    let (mut long, mut exp) = sci_digits(x, 40);
    if long.as_bytes()[17] == b'5' && long[18..].bytes().all(|b| b == b'0') {
        // looks like a tie, but the 40-digit form may itself be rounded: use the full exact
        // expansion (a double has at most 767 significant digits)
        (long, exp) = sci_digits(x, 800);
    }
    // ties and above round up (away from zero)
    let mut round_up = long.as_bytes()[17] >= b'5';
    let mut d: Vec<u8> = long.as_bytes()[..17].to_vec();
    let mut i = 17;
    while round_up && i > 0 {
        i -= 1;
        if d[i] == b'9' {
            d[i] = b'0';
        } else {
            d[i] += 1;
            round_up = false;
        }
    }
    if round_up {
        d.insert(0, b'1');
        d.truncate(17);
        exp += 1;
    }
    (String::from_utf8(d).unwrap(), exp)
}

/// LuaJIT `string.format("%.17g", x)` for finite x.
pub fn format_g17(x: f64) -> String {
    if x == 0.0 {
        return if x.is_sign_negative() {
            "-0".into()
        } else {
            "0".into()
        };
    }
    let neg = x < 0.0;
    let (digits, exp) = digits17(x);
    let mut s = String::new();
    if neg {
        s.push('-');
    }
    if !(-4..17).contains(&exp) {
        let frac = digits[1..].trim_end_matches('0');
        s.push_str(&digits[..1]);
        if !frac.is_empty() {
            s.push('.');
            s.push_str(frac);
        }
        s.push_str(&format!(
            "e{}{:02}",
            if exp < 0 { '-' } else { '+' },
            exp.abs()
        ));
    } else if exp >= 0 {
        let e = exp as usize + 1;
        s.push_str(&digits[..e]);
        let frac = digits[e..].trim_end_matches('0');
        if !frac.is_empty() {
            s.push('.');
            s.push_str(frac);
        }
    } else {
        s.push_str("0.");
        for _ in 0..(-exp - 1) {
            s.push('0');
        }
        s.push_str(digits.trim_end_matches('0'));
    }
    s
}

/// gm.json encode_string: escapes `"`, `\`, and control characters (Lua `%c`: 0-31 and 127).
fn encode_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 32 || c as u32 == 127 => {
                out.push_str(&format!("\\u{:04x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
#[allow(clippy::excessive_precision)]
mod tests {
    use super::format_g17;

    #[test]
    fn g17_matches_printf() {
        // reference strings from C printf("%.17g")
        assert_eq!(format_g17(59.057214604018057), "59.057214604018057");
        assert_eq!(format_g17(46335.456388888888), "46335.456388888888");
        assert_eq!(format_g17(0.1), "0.10000000000000001");
        assert_eq!(format_g17(2.56), "2.5600000000000001");
        assert_eq!(format_g17(-790.5), "-790.5");
        assert_eq!(format_g17(1e15), "1000000000000000");
        assert_eq!(format_g17(1e17), "1e+17");
        assert_eq!(format_g17(1.5e-5), "1.5e-05");
        assert_eq!(format_g17(0.00012), "0.00012");
        assert_eq!(format_g17(-3.25e20), "-3.25e+20");
        // exact tie at the 18th digit: LuaJIT rounds away from zero (C printf would give ...562)
        assert_eq!(format_g17(879.090240478515625), "879.09024047851563");
        assert_eq!(format_g17(9.99999999999999999e22), "9.9999999999999992e+22");
    }
}
