//! Right mouse button (the owner's change): in mushroom placement mode it cancels the mode, as
//! the original's Begin Step does; otherwise a right click in the world places a node directly
//! (`Command::QuickNode` at the cursor's world point, the old "ADD NODE" context menu is gone).
//! Over the HUD or while the left button is held (aiming a shot) it does nothing.

/// What a right click does this step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RmbAction {
    None,
    /// leave placement mode / close the build bar (`hud_build_close`)
    CancelPlacement,
    /// place a node at this world point
    QuickNode {
        x: f64,
        y: f64,
    },
}

/// Route a right click. `placement`: a mushroom type is picked or the build bar is open;
/// `over_ui`: obj_hud's `ui_capture_mouse` (HUD, menus, popups); `world`: the cursor's world
/// point through the zoomed camera (`None` outside the window).
pub fn rmb_route(
    rclick: bool,
    lmb_down: bool,
    placement: bool,
    over_ui: bool,
    world: Option<(f64, f64)>,
) -> RmbAction {
    if !rclick {
        return RmbAction::None;
    }
    if placement {
        return RmbAction::CancelPlacement;
    }
    if lmb_down || over_ui {
        return RmbAction::None;
    }
    match world {
        Some((x, y)) => RmbAction::QuickNode { x, y },
        None => RmbAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes() {
        let w = Some((10.0, 20.0));
        assert_eq!(rmb_route(false, false, false, false, w), RmbAction::None);
        assert_eq!(
            rmb_route(true, false, false, false, w),
            RmbAction::QuickNode { x: 10.0, y: 20.0 }
        );
        assert_eq!(
            rmb_route(true, false, true, false, w),
            RmbAction::CancelPlacement
        );
        assert_eq!(
            rmb_route(true, true, true, true, w),
            RmbAction::CancelPlacement
        );
        assert_eq!(rmb_route(true, false, false, true, w), RmbAction::None);
        assert_eq!(rmb_route(true, true, false, false, w), RmbAction::None);
        assert_eq!(rmb_route(true, false, false, false, None), RmbAction::None);
    }
}
