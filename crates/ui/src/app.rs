//! Root component: seeds the RNG, provides shared context, and switches between
//! the connection screens, the force-selection screen, and the battlefield.

use leptos::prelude::*;

use game::catalog::available_units;
use game::dice::XorShiftDice;
use game::scenario;
use game::state::GameState;

use super::battlefield::Battlefield;
use super::force_select::ForceSelect;
use super::hud::Hud;
use super::lobby::{Lobby, ModeSelect};
use super::net::Net;
use super::unit_card::UnitCard;
use super::{Catalog, ForceBuilder, Forces, Game, Overheat, Screen, ScreenKind, SelectedCard};

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
    provide_context(Screen(RwSignal::new(ScreenKind::ModeSelect)));
    provide_context(ForceBuilder(RwSignal::new(Forces::default())));
    provide_context(Catalog(StoredValue::new(available_units())));

    let net = Net::new();
    provide_context(net);

    let screen_ctx = expect_context::<Screen>();
    let screen = screen_ctx.0;

    // Pump the multiplayer socket on a short interval so peer connect/disconnect
    // events and inbound state snapshots are applied to the game. Cheap no-op
    // until a socket is opened (Local play never opens one).
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        let cb = Closure::<dyn FnMut()>::new(move || net.poll(game, screen_ctx));
        if let Some(win) = web_sys::window() {
            let _ = win.set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                100,
            );
        }
        // Keep the closure alive for the lifetime of the app.
        cb.forget();
    }

    view! {
        {move || match screen.get() {
            ScreenKind::ModeSelect => view! { <ModeSelect /> }.into_any(),
            ScreenKind::Lobby => view! { <Lobby /> }.into_any(),
            ScreenKind::ForceSelect => view! { <ForceSelect /> }.into_any(),
            ScreenKind::Battle => view! {
                <div class="app">
                    <Battlefield />
                    <Hud />
                </div>
                <UnitCard />
            }
            .into_any(),
        }}
    }
}
