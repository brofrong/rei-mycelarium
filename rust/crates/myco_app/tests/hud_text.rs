//! Task 17: the 14 px text minimum keeps the layout (own binary: it sets the global GUI scale).

use bevy::prelude::*;
use myco_app::assets::GameAssets;
use myco_app::ui::gm_draw::{DrawItem, Gm, UiFont};
use myco_app::ui::hud::{eff_scale, hud_icon_number, HudState};
use myco_app::ui::{px_per_unit, set_px_per_unit};
use std::path::Path;

fn assets() -> Option<GameAssets> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../build/assets");
    if !root.join("sprites.json").exists() {
        eprintln!("skipped: no build/assets");
        return None;
    }
    Some(
        GameAssets::load(
            &root,
            |_| Handle::default(),
            |_| Handle::default(),
            |_| Handle::default(),
        )
        .unwrap(),
    )
}

fn x_span(items: &[DrawItem]) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for it in items {
        if let DrawItem::Quad { x, w, .. } = it {
            lo = lo.min(*x);
            hi = hi.max(x + w);
        }
    }
    (lo, hi)
}

#[test]
fn icon_number_stays_centred_when_text_is_enlarged() {
    let Some(assets) = assets() else {
        return;
    };
    let hud = HudState::new();
    for ppu in [1.0, 0.3] {
        set_px_per_unit(ppu);
        let mut g = Gm::new(&assets);
        g.font = UiFont::Gm("Font1".into());
        g.valign = 1;
        hud_icon_number(
            &mut g,
            &assets,
            &hud,
            "apple_bite",
            700.0,
            24.0,
            0.4,
            "1234",
            0.5,
        );
        let (lo, hi) = x_span(&g.items);
        let mid = (lo + hi) / 2.0;
        assert!(
            (mid - 700.0).abs() < 3.0,
            "ppu {ppu}: group {lo}..{hi} not centred on 700"
        );
        if ppu < 1.0 {
            // the minimum really kicked in: the text is drawn larger than asked
            assert!(eff_scale(&g, 0.5) > 0.5);
            assert!(54.0 * eff_scale(&g, 0.5) * px_per_unit() >= 14.0 - 1e-9);
        }
    }
    set_px_per_unit(1.0);
}
