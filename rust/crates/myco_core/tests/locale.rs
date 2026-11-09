//! Localization against the LÖVE port (`rust/golden/data/locale.json`, from
//! `rust/golden/scripts/locale.lua`). The CSV is an asset: it is read from `build/assets`.

mod common;

use common::golden;
use myco_core::locale::{self, csv_split, FontId, Lang, Locale};
use std::path::PathBuf;

fn asset_csv() -> String {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../build/assets/localization.csv");
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e} (extract the game assets into build/assets)",
            path.display()
        )
    })
}

#[test]
fn locale_matches_love() {
    let g = golden("locale");
    let mut l = Locale::load(&asset_csv());
    let langs: Vec<&str> = l.langs().iter().map(|x| x.code()).collect();
    let want: Vec<&str> = g["langs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(langs, want);
    assert_eq!(want.len(), 12);
    let keys: Vec<&str> = g["keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(keys.len(), 30);
    assert!(keys.contains(&"button_add_node") && keys.contains(&"hint_no_nodes_in_water"));
    for code in &want {
        let lang = Lang::from_code(code).unwrap();
        l.set_language(lang);
        assert_eq!(l.language(), lang);
        for k in &keys {
            assert_eq!(
                l.loc(k),
                g["strings"][code][k].as_str().unwrap(),
                "{code}.{k}"
            );
        }
        let font = g["fonts"][code].as_str().unwrap();
        assert_eq!(
            FontId::for_lang(lang).file().unwrap_or(""),
            font,
            "font {code}"
        );
        assert_eq!(l.font_for(), FontId::for_lang(lang));
    }
}

#[test]
fn lang_codes_round_trip() {
    assert_eq!(Lang::ALL.len(), 12);
    for l in Lang::ALL {
        assert_eq!(Lang::from_code(l.code()), Some(l));
    }
    assert_eq!(Lang::from_code("xx"), None);
}

#[test]
fn csv_split_like_gml() {
    assert_eq!(csv_split("a,b,,c", b','), ["a", "b", "", "c"]);
    assert_eq!(
        csv_split("\"x, y\",\"say \"\"hi\"\"\",z", b','),
        ["x, y", "say \"hi\"", "z"]
    );
    assert_eq!(csv_split("a;b", b';'), ["a", "b"]);
    assert_eq!(csv_split("", b','), [""]);
    assert_eq!(csv_split("a,", b','), ["a", ""]);
    // an unclosed quote swallows the end of the line, so no last cell (as in GML)
    assert_eq!(csv_split("a,\"b", b','), ["a"]);
}

#[test]
fn load_csv_edge_cases() {
    let csv = "\u{feff}key ; context ; en ; ru\r\n\
               hud_food;top;FOOD;ЕДА\r\n\
               multi;x;\"line one\r\nline two\";\"раз\\nдва\"\r\n\
               only_en;x;Only;\r\n\
               ;x;skipped;skipped\r\n";
    let mut l = Locale::load(csv);
    assert_eq!(l.langs(), [Lang::En, Lang::Ru]);
    assert_eq!(l.loc("hud_food"), "FOOD");
    assert_eq!(l.loc("multi"), "line one\nline two");
    // English fallback struct (locale_en) when the CSV lacks a key
    assert_eq!(l.loc("debug_title"), "BALANCE (F1 to close)");
    assert_eq!(l.loc("missing"), "missing");
    l.set_language(Lang::Ru);
    assert_eq!(l.loc("hud_food"), "ЕДА");
    assert_eq!(l.loc("multi"), "раз\nдва");
    assert_eq!(l.loc("only_en"), "Only");
    // a language without a column falls back to English
    l.set_language(Lang::Ja);
    assert_eq!(l.language(), Lang::En);
    assert_eq!(l.font_for(), FontId::Default);
    assert_eq!(l.font_small_for(), None);
}

#[test]
fn language_with_one_key_is_not_available() {
    let l = Locale::load("key,en,de,fr\nk,a,b,\nk2,a,b,c\n");
    // fr has a single string: locale_available wants more than one
    assert_eq!(l.langs(), [Lang::En, Lang::De]);
}

#[test]
fn system_language_cycle_and_init() {
    let csv = asset_csv();
    let mut l = Locale::load(&csv);
    assert_eq!(l.system("pt", "br"), Lang::PtBr);
    assert_eq!(l.system("zh", "tw"), Lang::ZhHant);
    assert_eq!(l.system("zh", "HK"), Lang::ZhHant);
    assert_eq!(l.system("zh", "CN"), Lang::ZhHans);
    assert_eq!(l.system("ru", "RU"), Lang::Ru);
    assert_eq!(l.system("xx", ""), Lang::En);
    assert_eq!(l.cycle(-1), Lang::Ja);
    assert_eq!(l.cycle(1), Lang::En);
    assert_eq!(l.cycle(2), Lang::Fr);
    assert_eq!(l.font_small_for(), Some(FontId::CyrGerSpaPort));
    assert_eq!(Locale::init(&csv, "", "ko", "KR").language(), Lang::Ko);
    assert_eq!(Locale::init(&csv, "de", "ko", "KR").language(), Lang::De);
    assert_eq!(Locale::init(&csv, "bogus", "ko", "KR").language(), Lang::En);
}

#[test]
fn language_setting_in_settings_ini() {
    let dir = std::env::temp_dir().join(format!("myco_locale_ini_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    assert_eq!(locale::read_language(&dir), "");
    std::fs::write(
        dir.join("settings.ini"),
        "[display]\r\nfullscreen=\"1.000000\"\r\n",
    )
    .unwrap();
    locale::write_language(&dir, "uk").unwrap();
    assert_eq!(locale::read_language(&dir), "uk");
    let text = std::fs::read_to_string(dir.join("settings.ini")).unwrap();
    assert_eq!(
        text,
        "[display]\r\nfullscreen=\"1.000000\"\r\n[game]\r\nlanguage=\"uk\"\r\n"
    );
    locale::write_language(&dir, "ja").unwrap();
    let text = std::fs::read_to_string(dir.join("settings.ini")).unwrap();
    assert_eq!(
        text,
        "[display]\r\nfullscreen=\"1.000000\"\r\n[game]\r\nlanguage=\"ja\"\r\n"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}
