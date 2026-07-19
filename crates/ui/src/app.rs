//! Root component: seeds the RNG, provides shared context, and switches between
//! the pre-game force-selection screen and the battlefield.

use leptos::prelude::*;

use game::catalog::available_units;
use game::dice::XorShiftDice;
use game::scenario;
use game::state::GameState;

use super::battlefield::Battlefield;
use super::force_select::ForceSelect;
use super::hud::Hud;
use super::unit_card::UnitCard;
use super::{
    Catalog, ForceBuilder, Forces, Game, Overheat, Screen, ScreenKind, SelectedCard,
};

#[component]
pub fn App() -> impl IntoView {
    // Seed the deterministic PRNG from the browser's RNG so each session differs.
    let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;

    // The game starts with no units in a Deployment shell; it is replaced with
    // the real forces when the player leaves the force-selection screen.
    let game: Game = RwSignal::new(GameState::new_deploying(
        Vec::new(),
        scenario::demo_terrain(),
        XorShiftDice::new(seed),
    ));
    provide_context(game);
    provide_context(Overheat(RwSignal::new(false)));
    provide_context(SelectedCard(RwSignal::new(None)));
    provide_context(Screen(RwSignal::new(ScreenKind::ForceSelect)));
    provide_context(ForceBuilder(RwSignal::new(Forces::default())));
    provide_context(Catalog(StoredValue::new(available_units())));

    let screen = expect_context::<Screen>().0;

    view! {
        <Show
            when=move || screen.get() == ScreenKind::Battle
            fallback=|| view! { <ForceSelect /> }
        >
            <div class="app">
                <Battlefield />
                <Hud />
            </div>
            <UnitCard />
        </Show>
    }
}
