//! Leptos (client-side) presentation layer.
//!
//! Everything reactive lives behind a single [`Game`] signal placed in context
//! by [`app::App`]; child components read and mutate it. The DOM works in
//! pixels while the game logic works in board inches, so every coordinate is
//! converted through [`SCALE`].

use leptos::prelude::*;

use game::dice::XorShiftDice;
use game::state::GameState;
use game::unit::UnitCard as UnitCardData;

pub mod app;
pub mod battlefield;
pub mod force_select;
pub mod hud;
pub mod lobby;
pub mod net;
pub mod overlays;
pub mod unit_card;
pub mod unit_view;

pub use app::App;

/// The single source of truth for the whole game, shared via context.
pub type Game = RwSignal<GameState<XorShiftDice>>;

/// Pixels per board inch — converts the inch-based domain to on-screen size.
pub const SCALE: f64 = 12.0;

/// Which top-level screen the app is showing.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScreenKind {
    /// Opening chooser: Host / Join / Local.
    ModeSelect,
    /// Online waiting room (host shows its room id; joiner enters one).
    Lobby,
    /// Pre-game force builder (both players pick units).
    ForceSelect,
    /// The board: deployment and the turn loop.
    Battle,
}

/// The active screen, shared via context.
#[derive(Clone, Copy)]
pub struct Screen(pub RwSignal<ScreenKind>);

/// One unit chosen into a player's force, with its (adjustable) pilot skill and
/// display colour.
#[derive(Clone, PartialEq)]
pub struct Pick {
    pub card: UnitCardData,
    pub pilot_skill: i32,
    pub color: String,
}

/// Both players' in-progress forces (index = player id).
#[derive(Clone, Default, PartialEq)]
pub struct Forces {
    pub players: [Vec<Pick>; 2],
}

/// The force-builder state, shared via context.
#[derive(Clone, Copy)]
pub struct ForceBuilder(pub RwSignal<Forces>);

/// The master unit list, parsed once and shared via context.
#[derive(Clone, Copy)]
pub struct Catalog(pub StoredValue<Vec<UnitCardData>>);

/// UI-only flag: whether the next attack should spend heat for bonus damage.
/// Wrapped in a newtype so it is unambiguous in the context store.
#[derive(Clone, Copy)]
pub struct Overheat(pub RwSignal<bool>);

/// UI-only state: the id of the unit whose full Alpha Strike card is being shown
/// in the detail modal, or `None` when the modal is closed. Wrapped in a newtype
/// so it is unambiguous in the context store.
#[derive(Clone, Copy)]
pub struct SelectedCard(pub RwSignal<Option<usize>>);

/// Read the shared game signal from context.
pub fn use_game() -> Game {
    expect_context::<Game>()
}

/// Read the overheat toggle from context.
pub fn use_overheat() -> RwSignal<bool> {
    expect_context::<Overheat>().0
}

/// Read the "which unit card is open" signal from context.
pub fn use_selected_card() -> RwSignal<Option<usize>> {
    expect_context::<SelectedCard>().0
}

/// Read the active-screen signal from context.
pub fn use_screen() -> RwSignal<ScreenKind> {
    expect_context::<Screen>().0
}

/// Read the force-builder signal from context.
pub fn use_forces() -> RwSignal<Forces> {
    expect_context::<ForceBuilder>().0
}

/// Read the shared master unit list from context.
pub fn use_catalog() -> StoredValue<Vec<UnitCardData>> {
    expect_context::<Catalog>().0
}
