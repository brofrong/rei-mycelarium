//! Localization: `localization.csv` (scr_locale), `loc(key)`, language choice and the font per
//! language. Text measuring and font fitting (`locale_font_fit`, `locale_font_ink_h`) belong to
//! the app; the core only says which font a language uses ([`FontId`]).

use std::collections::HashMap;
use std::io;
use std::path::Path;

/// The 12 languages of `localization.csv`, in `locale_available` order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lang {
    En,
    De,
    Fr,
    Es,
    It,
    PtBr,
    Ru,
    Uk,
    ZhHant,
    ZhHans,
    Ko,
    Ja,
}

impl Lang {
    pub const ALL: [Lang; 12] = [
        Lang::En,
        Lang::De,
        Lang::Fr,
        Lang::Es,
        Lang::It,
        Lang::PtBr,
        Lang::Ru,
        Lang::Uk,
        Lang::ZhHant,
        Lang::ZhHans,
        Lang::Ko,
        Lang::Ja,
    ];

    /// Column name in the CSV and value of `[game] language` in settings.ini.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::De => "de",
            Lang::Fr => "fr",
            Lang::Es => "es",
            Lang::It => "it",
            Lang::PtBr => "pt-BR",
            Lang::Ru => "ru",
            Lang::Uk => "uk",
            Lang::ZhHant => "zh-Hant",
            Lang::ZhHans => "zh-Hans",
            Lang::Ko => "ko",
            Lang::Ja => "ja",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code() == code)
    }
}

/// UI font: `Font1` (the game's own font) or a TTF/OTF from `build/assets/fonts`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontId {
    Default,
    ChSim,
    Korean,
    Japan,
    CyrGerSpaPort,
}

impl FontId {
    // port: locale_font_file
    /// `locale_font_file(code)`.
    pub fn for_lang(lang: Lang) -> FontId {
        match lang {
            Lang::ZhHans | Lang::ZhHant => FontId::ChSim,
            Lang::Ko => FontId::Korean,
            Lang::Ja => FontId::Japan,
            Lang::De | Lang::Fr | Lang::Es | Lang::It | Lang::PtBr | Lang::Ru | Lang::Uk => {
                FontId::CyrGerSpaPort
            }
            Lang::En => FontId::Default,
        }
    }

    /// Asset path relative to the assets folder; `None` for `Font1`.
    pub fn file(self) -> Option<&'static str> {
        match self {
            FontId::Default => None,
            FontId::ChSim => Some("fonts/ch-sim.ttf"),
            FontId::Korean => Some("fonts/korean.otf"),
            FontId::Japan => Some("fonts/japan.ttf"),
            FontId::CyrGerSpaPort => Some("fonts/cyrillic-ger-spa-port.ttf"),
        }
    }
}

/// `___struct___114__locale_init__scr_locale`: English strings used when the CSV lacks a key.
const LOCALE_EN: [(&str, &str); 16] = [
    ("hud_food", "FOOD"),
    ("hud_water", "WATER"),
    ("hud_length", "LENGTH"),
    ("hud_points", "POINTS"),
    ("hud_wind", "WIND"),
    ("hud_cm", "cm"),
    ("hud_in", "in"),
    ("wind_tooltip", "Wind strength and direction"),
    ("points_tooltip", "Upgrade points"),
    ("points_till_next", "till next point"),
    ("hud_next_point", "next in"),
    ("upg_parasitism_yield", "Consumptor Yield"),
    (
        "upg_parasitism_yield_desc",
        "More food from everything consumptor mushrooms eat.",
    ),
    ("button_language", "LANGUAGE"),
    ("language_name", "ENGLISH"),
    ("debug_title", "BALANCE (F1 to close)"),
];

/// Lua `%s` (gm.string_trim).
fn gml_trim(s: &str) -> &str {
    s.trim_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c'])
}

// port: locale_csv_split
/// `locale_csv_split(line, delim)`: split on `delim` outside double quotes (bytewise); a cell
/// wrapped in quotes loses them and `""` becomes `"`.
pub fn csv_split(line: &str, delim: u8) -> Vec<String> {
    let src = line.as_bytes();
    let n = src.len();
    let mut out = Vec::new();
    let mut inq = false;
    let mut start = 0;
    for i in 0..=n {
        let c = if i >= n { None } else { Some(src[i]) };
        if inq {
            if c == Some(b'"') {
                inq = false;
            }
        } else if c == Some(b'"') {
            inq = true;
        } else if c == Some(delim) || c.is_none() {
            // buffer_peek(buffer_string) stops at a NUL byte
            let raw = &src[start..i];
            let raw = &raw[..raw.iter().position(|&b| b == 0).unwrap_or(raw.len())];
            let mut cell = String::from_utf8_lossy(raw).into_owned();
            let chars = cell.chars().count();
            if chars >= 2 && cell.starts_with('"') && cell.ends_with('"') {
                cell = cell[1..cell.len() - 1].replace("\"\"", "\"");
            }
            out.push(cell);
            start = i + 1;
        }
    }
    out
}

/// Loaded `localization.csv` plus the current language (`global.locale_*`).
#[derive(Clone, Debug)]
pub struct Locale {
    table: HashMap<Lang, HashMap<String, String>>,
    langs: Vec<Lang>,
    id: Lang,
    strings: HashMap<String, String>,
    font: FontId,
}

impl Locale {
    /// `locale_load_csv` + `locale_available`, language English.
    pub fn load(csv: &str) -> Locale {
        let table = load_csv(csv);
        let mut l = Locale {
            langs: available(&table),
            table,
            id: Lang::En,
            strings: HashMap::new(),
            font: FontId::Default,
        };
        l.set_language(Lang::En);
        l
    }

    // port: locale_init
    /// `locale_init`: load the CSV, then the language saved in settings.ini (`saved`, empty
    /// when unset) or else the system one (`os_lang`/`os_region` as `os_get_language/region`).
    pub fn init(csv: &str, saved: &str, os_lang: &str, os_region: &str) -> Locale {
        let mut l = Locale::load(csv);
        let lang = if saved.is_empty() {
            l.system(os_lang, os_region)
        } else {
            Lang::from_code(saved).unwrap_or(Lang::En)
        };
        l.set_language(lang);
        l
    }

    /// `global.locale_langs`.
    pub fn langs(&self) -> &[Lang] {
        &self.langs
    }

    /// `global.locale_id`.
    pub fn language(&self) -> Lang {
        self.id
    }

    // port: locale_system
    /// `locale_system()`: `pt` → `pt-BR`, `zh` → `zh-Hant` for TW/HK/MO else `zh-Hans`;
    /// English when the result is not available.
    pub fn system(&self, os_lang: &str, os_region: &str) -> Lang {
        let region = os_region.to_uppercase();
        let code = match os_lang {
            "pt" => "pt-BR",
            "zh" if matches!(region.as_str(), "TW" | "HK" | "MO") => "zh-Hant",
            "zh" => "zh-Hans",
            other => other,
        };
        match Lang::from_code(code) {
            Some(l) if self.langs.contains(&l) => l,
            _ => Lang::En,
        }
    }

    // port: locale_set
    /// `locale_set(code)`: English strings, then the CSV's English column, then the language's
    /// column on top. A language that is not available falls back to English.
    pub fn set_language(&mut self, lang: Lang) {
        let lang = if self.langs.contains(&lang) {
            lang
        } else {
            Lang::En
        };
        self.id = lang;
        self.font = locale_font_load(lang);
        let mut strings: HashMap<String, String> = LOCALE_EN
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let chain: &[Lang] = if lang == Lang::En {
            &[Lang::En]
        } else {
            &[Lang::En, lang]
        };
        for l in chain {
            if let Some(t) = self.table.get(l) {
                for (k, v) in t {
                    strings.insert(k.clone(), v.clone());
                }
            }
        }
        self.strings = strings;
    }

    /// Mix in extra strings (the Rust port's `locale_extra.csv`, ruling R4): keys the main CSV
    /// lacks are added per language; keys it already has are kept. Missing cells fall back to
    /// English like any other key. The language list and the current language stay as they are.
    pub fn extend_from_csv(&mut self, csv: &str) {
        for (lang, extra) in load_csv(csv) {
            let t = self.table.entry(lang).or_default();
            for (k, v) in extra {
                t.entry(k).or_insert(v);
            }
        }
        self.set_language(self.id);
    }

    // port: locale_cycle
    /// `locale_cycle(dir)`: the next/previous available language. The caller persists it with
    /// [`write_language`] (the original writes `[game] language` right away).
    pub fn cycle(&mut self, dir: i32) -> Lang {
        let n = self.langs.len() as i64;
        let cur = self.langs.iter().rposition(|&l| l == self.id).unwrap_or(0) as i64;
        let next = (cur + dir as i64 + n).rem_euclid(n) as usize;
        self.set_language(self.langs[next]);
        self.id
    }

    // port: loc
    /// `loc(key)`: the string, or the key itself when there is none.
    pub fn loc<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).map(String::as_str).unwrap_or(key)
    }

    // port: ui_font
    /// `ui_font()`: the language's font (the app falls back to `Font1` when it cannot load it).
    pub fn font_for(&self) -> FontId {
        self.font
    }

    // port: ui_font_small
    /// `ui_font_small()`: the small variant of the language's font, `None` (`-1`) for `Font1`.
    pub fn font_small_for(&self) -> Option<FontId> {
        match self.font {
            FontId::Default => None,
            f => Some(f),
        }
    }
}

// port: locale_font_load
/// `locale_font_load(code)`: pick the font for the language. Loading and fitting the font file
/// at the right size (`locale_font_fit`) is the app's job.
fn locale_font_load(lang: Lang) -> FontId {
    FontId::for_lang(lang)
}

// port: locale_load_csv
/// `locale_load_csv`: lang → key → string. The first line is the header (BOM stripped,
/// `;` is the delimiter when it outnumbers `,`); rows continue over line breaks while their
/// quote count is odd; empty cells are skipped; `\n` in a cell becomes a line break.
fn load_csv(text: &str) -> HashMap<Lang, HashMap<String, String>> {
    let mut table = HashMap::new();
    let text = text.replace("\r\n", "\n");
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut buf = String::new();
    let mut delim = b',';
    let mut first = true;
    for line in lines {
        let mut line = line;
        if first {
            if let Some(rest) = line.strip_prefix('\u{feff}') {
                line = rest;
            }
            if line.matches(';').count() > line.matches(',').count() {
                delim = b';';
            }
            first = false;
        }
        if buf.is_empty() {
            buf = line.to_string();
        } else {
            buf.push('\n');
            buf.push_str(line);
        }
        if buf.matches('"').count().is_multiple_of(2) {
            rows.push(csv_split(&buf, delim));
            buf.clear();
        }
    }
    let Some(header) = rows.first() else {
        return table;
    };
    let header: Vec<String> = header.iter().map(|h| gml_trim(h).to_string()).collect();
    let Some(key_col) = header.iter().rposition(|h| h == "key") else {
        return table;
    };
    // columns that become languages (unknown language codes are not supported: Lang is closed)
    let col_lang: Vec<Option<Lang>> = header
        .iter()
        .enumerate()
        .map(|(i, h)| {
            if i != key_col && h != "context" {
                Lang::from_code(h)
            } else {
                None
            }
        })
        .collect();
    for lang in col_lang.iter().flatten() {
        table.entry(*lang).or_insert_with(HashMap::new);
    }
    for row in &rows[1..] {
        if key_col >= row.len() {
            continue;
        }
        let key = gml_trim(&row[key_col]);
        if key.is_empty() {
            continue;
        }
        for c in 0..row.len().min(header.len()) {
            if let Some(lang) = col_lang[c] {
                if !row[c].is_empty() {
                    table
                        .get_mut(&lang)
                        .unwrap()
                        .insert(key.to_string(), row[c].replace("\\n", "\n"));
                }
            }
        }
    }
    table
}

// port: locale_available
/// `locale_available()`: English, then every other language (in [`Lang::ALL`] order) whose
/// column holds more than one string.
fn available(table: &HashMap<Lang, HashMap<String, String>>) -> Vec<Lang> {
    let mut out = vec![Lang::En];
    for l in &Lang::ALL[1..] {
        if table.get(l).is_some_and(|t| t.len() > 1) {
            out.push(*l);
        }
    }
    out
}

/// `ini_read_string("game", "language", "")` from `<dir>/settings.ini`.
pub fn read_language(dir: &Path) -> String {
    let text = std::fs::read(dir.join("settings.ini")).unwrap_or_default();
    crate::save::ini_read(&String::from_utf8_lossy(&text), "game", "language").unwrap_or_default()
}

/// `ini_write_string("game", "language", code)`, keeping the rest of settings.ini.
pub fn write_language(dir: &Path, code: &str) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join("settings.ini");
    let text = std::fs::read(&path).unwrap_or_default();
    let out = crate::save::ini_write(&String::from_utf8_lossy(&text), "game", "language", code);
    let tmp = dir.join("settings.ini.tmp");
    std::fs::write(&tmp, out)?;
    std::fs::rename(&tmp, &path)
}
