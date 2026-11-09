//! Skill tree layout (`global.upgrade_defs` from `upgrades_init`, scr_upgrades): the upgrade
//! definitions with their tree positions and links. Logic reads costs and keys from here; the
//! tree screen (Task 19) reads positions, links and icons.

use crate::balance::vis;
use crate::gml;

/// `def.kind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UpgKind {
    /// the tree's root (`origin`): owned from the start, never bought
    Root,
    /// bought several times; `upg_bonus` = level * `bal(step_key)`
    Level,
    /// bought once for `bal(cost_key)`
    Unlock,
    /// shown but never buyable (no definition uses it, the code handles it)
    Ghost,
}

impl UpgKind {
    pub fn as_str(self) -> &'static str {
        match self {
            UpgKind::Root => "root",
            UpgKind::Level => "level",
            UpgKind::Unlock => "unlock",
            UpgKind::Ghost => "ghost",
        }
    }
}

/// `def.trunk`: the branch of the tree a node belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trunk {
    /// the root (`""`)
    None,
    Hyphae,
    Enzymes,
    Fruiting,
}

impl Trunk {
    pub fn as_str(self) -> &'static str {
        match self {
            Trunk::None => "",
            Trunk::Hyphae => "hyphae",
            Trunk::Enzymes => "enzymes",
            Trunk::Fruiting => "fruiting",
        }
    }
}

/// One upgrade definition (`___struct___248..279__upgrades_init`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeNode {
    pub id: &'static str,
    pub trunk: Trunk,
    pub kind: UpgKind,
    /// parent id; `""` for the root
    pub parent: &'static str,
    /// parent level needed (`def.req`)
    pub req: f64,
    /// cost tier (`def.tier`)
    pub tier: f64,
    /// layout position before `tree_layout_scale_*` (`def.px`, `def.py`)
    pub px: f64,
    pub py: f64,
    /// the bend of the link from the parent (`def.bx`, `def.by`), when the node has one
    pub via: Option<(f64, f64)>,
    pub available: bool,
    /// sprite name of the node icon (`def.icon`); `None` for -1 (level nodes draw their own)
    pub icon: Option<&'static str>,
    /// balance key of the max level (`Level` only, else `""`)
    pub max_key: &'static str,
    /// balance key of the per-level bonus (`Level` only, else `""`)
    pub step_key: &'static str,
    /// balance key of the price (`Unlock` only, else `""`)
    pub cost_key: &'static str,
}

const fn node(
    id: &'static str,
    trunk: Trunk,
    kind: UpgKind,
    parent: &'static str,
    tier: f64,
    px: f64,
    py: f64,
) -> TreeNode {
    TreeNode {
        id,
        trunk,
        kind,
        parent,
        req: 1.0,
        tier,
        px,
        py,
        via: None,
        available: true,
        icon: None,
        max_key: "",
        step_key: "",
        cost_key: "",
    }
}

#[allow(clippy::too_many_arguments)]
const fn level(
    id: &'static str,
    trunk: Trunk,
    parent: &'static str,
    tier: f64,
    px: f64,
    py: f64,
    max_key: &'static str,
    step_key: &'static str,
) -> TreeNode {
    let mut n = node(id, trunk, UpgKind::Level, parent, tier, px, py);
    n.max_key = max_key;
    n.step_key = step_key;
    n
}

#[allow(clippy::too_many_arguments)]
const fn unlock(
    id: &'static str,
    trunk: Trunk,
    parent: &'static str,
    tier: f64,
    px: f64,
    py: f64,
    icon: Option<&'static str>,
    cost_key: &'static str,
) -> TreeNode {
    let mut n = node(id, trunk, UpgKind::Unlock, parent, tier, px, py);
    n.icon = icon;
    n.cost_key = cost_key;
    n
}

const fn with_via(mut n: TreeNode, bx: f64, by: f64) -> TreeNode {
    n.via = Some((bx, by));
    n
}

use Trunk::{Enzymes, Fruiting, Hyphae};

/// `global.upgrade_defs` in `upgrades_init` order (the order matters: `upg_children`, the
/// grow animation and the tree screen walk it).
static DEFS: [TreeNode; 32] = [
    TreeNode {
        id: "origin",
        trunk: Trunk::None,
        kind: UpgKind::Root,
        parent: "",
        req: 0.0,
        tier: -0.5,
        px: 0.0,
        py: 0.0,
        via: None,
        available: true,
        icon: Some("originshroom"),
        max_key: "",
        step_key: "",
        cost_key: "",
    },
    level(
        "shot_range",
        Hyphae,
        "origin",
        1.0,
        -775.0,
        103.0,
        "upg_shot_range_max_level",
        "upg_shot_range_per_level",
    ),
    level(
        "shot_width",
        Hyphae,
        "origin",
        2.0,
        -558.0,
        185.0,
        "upg_shot_width_max_level",
        "upg_shot_width_per_level",
    ),
    level(
        "water_efficiency",
        Hyphae,
        "origin",
        2.0,
        -288.0,
        245.0,
        "upg_water_efficiency_max_level",
        "upg_water_efficiency_per_level",
    ),
    level(
        "budding_chance",
        Hyphae,
        "origin",
        2.0,
        289.0,
        245.0,
        "upg_budding_chance_max_level",
        "upg_budding_chance_per_level",
    ),
    level(
        "node_efficiency",
        Hyphae,
        "origin",
        3.0,
        554.0,
        185.0,
        "upg_node_efficiency_max_level",
        "upg_node_efficiency_per_level",
    ),
    level(
        "soil_penetration",
        Hyphae,
        "origin",
        3.0,
        780.0,
        103.0,
        "upg_soil_penetration_max_level",
        "upg_soil_penetration_per_level",
    ),
    unlock(
        "unlock_absorption",
        Enzymes,
        "origin",
        1.0,
        0.0,
        290.0,
        Some("leaf"),
        "upg_unlock_absorption_cost",
    ),
    level(
        "absorption_speed",
        Enzymes,
        "unlock_absorption",
        2.0,
        -161.0,
        405.0,
        "upg_absorption_speed_max_level",
        "upg_absorption_speed_per_level",
    ),
    level(
        "absorption_yield",
        Enzymes,
        "unlock_absorption",
        2.0,
        0.0,
        407.0,
        "upg_absorption_yield_max_level",
        "upg_absorption_yield_per_level",
    ),
    unlock(
        "unlock_protein",
        Enzymes,
        "unlock_absorption",
        3.0,
        166.0,
        405.0,
        None,
        "upg_unlock_protein_cost",
    ),
    with_via(
        unlock(
            "unlock_repeller",
            Fruiting,
            "origin",
            2.0,
            -820.0,
            -118.0,
            Some("explosiveshroom"),
            "upg_unlock_repeller_cost",
        ),
        -406.0,
        40.0,
    ),
    level(
        "repeller_autoregen",
        Fruiting,
        "unlock_repeller",
        3.0,
        -904.0,
        -272.0,
        "upg_repeller_autoregen_max_level",
        "upg_repeller_autoregen_per_level",
    ),
    level(
        "repeller_regen_cost",
        Fruiting,
        "unlock_repeller",
        3.0,
        -739.0,
        -275.0,
        "upg_repeller_regen_cost_max_level",
        "upg_repeller_regen_cost_per_level",
    ),
    unlock(
        "unlock_predator",
        Fruiting,
        "origin",
        1.0,
        -408.0,
        -113.0,
        Some("predatorshroom"),
        "upg_unlock_predator_cost",
    ),
    level(
        "predator_digestion",
        Fruiting,
        "unlock_predator",
        2.0,
        -486.0,
        -275.0,
        "upg_predator_digestion_max_level",
        "upg_predator_digestion_per_level",
    ),
    level(
        "predator_yield",
        Fruiting,
        "unlock_predator",
        2.0,
        -328.0,
        -275.0,
        "upg_predator_yield_max_level",
        "upg_predator_yield_per_level",
    ),
    unlock(
        "unlock_tree",
        Fruiting,
        "origin",
        2.0,
        407.0,
        -105.0,
        Some("treeshroom"),
        "upg_unlock_tree_cost",
    ),
    level(
        "tree_water_yield",
        Fruiting,
        "unlock_tree",
        3.0,
        328.0,
        -272.0,
        "upg_tree_water_yield_max_level",
        "upg_tree_water_yield_per_level",
    ),
    level(
        "tree_drop_rate",
        Fruiting,
        "unlock_tree",
        3.0,
        488.0,
        -272.0,
        "upg_tree_drop_rate_max_level",
        "upg_tree_drop_rate_per_level",
    ),
    with_via(
        unlock(
            "unlock_parasitism",
            Enzymes,
            "origin",
            3.0,
            820.0,
            -105.0,
            Some("consumptorshroom"),
            "upg_unlock_parasitism_cost",
        ),
        406.0,
        40.0,
    ),
    level(
        "parasitism_speed",
        Enzymes,
        "unlock_parasitism",
        4.0,
        741.0,
        -272.0,
        "upg_parasitism_speed_max_level",
        "upg_parasitism_speed_per_level",
    ),
    level(
        "parasitism_yield",
        Enzymes,
        "unlock_parasitism",
        4.0,
        901.0,
        -272.0,
        "upg_parasitism_yield_max_level",
        "upg_parasitism_yield_per_level",
    ),
    unlock(
        "unlock_spore",
        Fruiting,
        "origin",
        1.0,
        0.0,
        -178.0,
        Some("sporeshroom"),
        "upg_unlock_spore_cost",
    ),
    level(
        "spore_count",
        Fruiting,
        "unlock_spore",
        2.0,
        -126.0,
        -345.0,
        "upg_spore_count_max_level",
        "upg_spore_count_per_level",
    ),
    level(
        "spore_quality",
        Fruiting,
        "unlock_spore",
        2.0,
        0.0,
        -345.0,
        "upg_spore_quality_max_level",
        "upg_spore_quality_per_level",
    ),
    level(
        "spore_lightness",
        Fruiting,
        "unlock_spore",
        3.0,
        127.0,
        -345.0,
        "upg_spore_lightness_max_level",
        "upg_spore_lightness_per_level",
    ),
    unlock(
        "unlock_experience",
        Fruiting,
        "origin",
        3.0,
        0.0,
        -520.0,
        Some("experienceshroom"),
        "upg_unlock_experience_cost",
    ),
    unlock(
        "unlock_digger",
        Hyphae,
        "origin",
        4.0,
        -397.0,
        -467.0,
        Some("diggershroom"),
        "upg_unlock_digger_cost",
    ),
    level(
        "digger_length",
        Hyphae,
        "unlock_digger",
        5.0,
        -397.0,
        -632.0,
        "upg_digger_length_max_level",
        "upg_digger_length_per_level",
    ),
    unlock(
        "unlock_stinky",
        Fruiting,
        "origin",
        4.0,
        397.0,
        -467.0,
        Some("stinkyshroom"),
        "upg_unlock_stinky_cost",
    ),
    level(
        "stinky_width",
        Fruiting,
        "unlock_stinky",
        5.0,
        397.0,
        -632.0,
        "upg_stinky_width_max_level",
        "upg_stinky_width_per_level",
    ),
];

/// Every upgrade definition in `global.upgrade_defs` order.
pub fn layout() -> &'static [TreeNode] {
    &DEFS
}

// port: upg_def
/// The definition with this id.
pub fn upg_def(id: &str) -> Option<&'static TreeNode> {
    DEFS.iter().find(|d| d.id == id)
}

// port: upg_tree_pos
/// Tree position of a node: `(px * tree_layout_scale_x, py * tree_layout_scale_y)`.
pub fn upg_tree_pos(def: &TreeNode) -> (f64, f64) {
    (
        def.px * vis("tree_layout_scale_x"),
        def.py * vis("tree_layout_scale_y"),
    )
}

// port: upg_tree_via
/// The scaled bend point of the link from the parent, when the node has one.
pub fn upg_tree_via(def: &TreeNode) -> Option<(f64, f64)> {
    let (bx, by) = def.via?;
    Some((
        bx * vis("tree_layout_scale_x"),
        by * vis("tree_layout_scale_y"),
    ))
}

// port: upg_children
/// The nodes whose parent is `id`, in definition order.
pub fn upg_children(id: &str) -> Vec<&'static TreeNode> {
    DEFS.iter().filter(|d| d.parent == id).collect()
}

/// `tree_hypha_length(id)` without built tree hyphae (the tree screen, Task 19, supplies the
/// drawn length `main_len * total` through `ProgressRt::tree_hypha_len`): 1.3 times the
/// distance from the parent's position, 1 for a node without a parent.
pub fn tree_hypha_length_fallback(id: &str) -> f64 {
    let Some(d) = upg_def(id) else {
        return 1.0;
    };
    let Some(par) = upg_def(d.parent) else {
        return 1.0;
    };
    let (ax, ay) = upg_tree_pos(par);
    let (bx, by) = upg_tree_pos(d);
    gml::point_distance(ax, ay, bx, by) * 1.3
}
