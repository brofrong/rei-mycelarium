//! What the simulation tells the app each frame: resource changes (for floating numbers),
//! refused node placements, sounds, saves and tutorial tips. Logic tasks extend [`SimEvent`]
//! with their own variants; the app drains them with `Sim::drain_events`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    Food,
    Water,
    UpgradePoints,
}

/// Why a node (right click / quick node) was not placed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeReject {
    NotOnMycelium,
    InWater,
    TooCloseToNode,
    NotEnoughFood { need: i64 },
    Blocked,
}

/// A sound to play. `name` is the sound's file stem in `build/assets/sounds` (as the original
/// passes it to `sfx_play` and friends); `x`/`y` are world coordinates for positional sounds.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundCue {
    pub name: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub gain: f64,
    pub pitch: f64,
}

impl SoundCue {
    /// A non-positional sound at full gain and normal pitch.
    pub fn new(name: impl Into<String>) -> SoundCue {
        SoundCue {
            name: name.into(),
            x: None,
            y: None,
            gain: 1.0,
            pitch: 1.0,
        }
    }

    pub fn at(mut self, x: f64, y: f64) -> SoundCue {
        self.x = Some(x);
        self.y = Some(y);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SimEvent {
    /// `source`: a stable id of what produced the change (one floating number per source/frame).
    Resource {
        kind: ResourceKind,
        delta: f64,
        x: f64,
        y: f64,
        source: u64,
    },
    NodeRejected {
        reason: NodeReject,
        x: f64,
        y: f64,
    },
    Sound(SoundCue),
    Saved,
    GameOver,
    TutorialTip {
        key: String,
    },
}
