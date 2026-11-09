//! Release builds must give the same bits as debug builds (and as LuaJIT). LLVM fuses
//! `sin(a)`/`cos(a)` into `__sincos_stret` on Apple targets and rewrites `pow` with constant
//! or integer exponents, which changes results by an ulp. Core therefore calls trig and `pow`
//! only through `gml::{sin, cos, tan, power}`, which hide their arguments with `black_box`.
//! This test keeps new direct calls out.

use std::path::Path;

const FORBIDDEN: &[&str] = &[
    ".sin()",
    ".cos()",
    ".tan()",
    ".sin_cos()",
    ".powf(",
    ".powi(",
    ".exp()",
    ".ln()",
    ".log10()",
    ".log2()",
    ".exp2()",
    ".exp_m1()",
    ".ln_1p()",
];

fn scan(dir: &Path, hits: &mut Vec<String>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            scan(&p, hits);
        } else if p.extension().is_some_and(|x| x == "rs") && !p.ends_with("gml.rs") {
            let src = std::fs::read_to_string(&p).unwrap();
            for (i, line) in src.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                for f in FORBIDDEN {
                    if code.contains(f) {
                        hits.push(format!("{}:{}: {}", p.display(), i + 1, line.trim()));
                    }
                }
            }
        }
    }
}

#[test]
fn core_calls_libm_only_through_gml() {
    let mut hits = Vec::new();
    scan(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut hits,
    );
    assert!(
        hits.is_empty(),
        "use gml::sin/cos/tan/power instead:\n{}",
        hits.join("\n")
    );
}
