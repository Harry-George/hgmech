//! Sidebar: phase control, the unit roster (cards), and the combat log.

use leptos::prelude::*;

use game::state::Phase;

use super::{use_game, use_overheat};

#[component]
pub fn Hud() -> impl IntoView {
    let game = use_game();
    let overheat = use_overheat();

    let phase_name = move || game.with(|g| g.phase.as_str());
    let button_label = move || game.with(|g| g.primary_action_label());

    let meta = move || {
        game.with(|g| match g.phase {
            Phase::Deployment => format!("Player {} placing a unit", g.current_player()),
            Phase::Movement | Phase::Attack => {
                format!("Turn {} • Player {} activating", g.turn, g.current_player())
            }
            _ => format!("Turn {}", g.turn),
        })
    };

    let initiative = move || {
        game.with(|g| {
            if g.initiative.is_empty() {
                String::new()
            } else {
                let parts: Vec<String> = g
                    .initiative
                    .iter()
                    .map(|(p, r)| format!("P{p}: {r}"))
                    .collect();
                format!("Initiative — {}", parts.join(", "))
            }
        })
    };

    let hint = move || {
        game.with(|g| match g.phase {
            Phase::Deployment => "Drag your highlighted unit into your home-edge zone, then Place Unit to hand off to your opponent.",
            Phase::Initiative => "Roll initiative to start the turn.",
            Phase::Movement => "Drag one highlighted unit to move within its MV (Undo to retry), then End Unit Move to pass to your opponent.",
            Phase::Attack => "Click one of your units, then click an enemy to fire. End Attacks to pass.",
            Phase::End => "Resolve the End phase to dissipate heat and begin the next turn.",
        })
    };

    let on_advance = move |_| game.update(|g| g.advance());
    let game_over = move || game.with(|g| g.winner.is_some());
    let winner_text =
        move || game.with(|g| g.winner.map(|p| format!("Player {p} wins!")).unwrap_or_default());

    let roster = move || {
        game.with(|g| {
            g.units
                .iter()
                .map(|u| {
                    let status = if u.is_destroyed() {
                        " — destroyed".to_string()
                    } else if u.shutdown {
                        " — shutdown".to_string()
                    } else {
                        String::new()
                    };
                    let line = format!(
                        "P{} {} ({}) — MV {:.0}\" • S/M/L {}/{}/{} • A/S {}/{} {}/{} • PS {}{}",
                        u.player,
                        u.card.name,
                        u.card.size.as_str(),
                        u.card.mv_inches,
                        u.card.damage.short,
                        u.card.damage.medium,
                        u.card.damage.long,
                        u.cur_armor,
                        u.card.armor,
                        u.cur_structure,
                        u.card.structure,
                        u.card.pilot_skill,
                        status,
                    );
                    view! { <div class="log__line">{line}</div> }
                })
                .collect::<Vec<_>>()
        })
    };

    let log = move || {
        game.with(|g| {
            g.log
                .iter()
                .map(|l| view! { <div class="log__line">{l.clone()}</div> })
                .collect::<Vec<_>>()
        })
    };

    view! {
        <div class="sidebar">
            <div class="panel">
                <h2>"Alpha Strike"</h2>
                <Show
                    when=game_over
                    fallback=move || {
                        view! {
                            <div class="hud__phase">{phase_name}" Phase"</div>
                            <div class="hud__meta">{meta}</div>
                            <Show when=move || !initiative().is_empty()>
                                <div class="hud__meta">{initiative}</div>
                            </Show>
                            <button class="btn" on:click=on_advance>{button_label}</button>
                            <Show when=move || game.with(|g| g.phase == Phase::Attack)>
                                <label class="toggle">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || overheat.get()
                                        on:change=move |e| overheat.set(event_target_checked(&e))
                                    />
                                    "Overheat next attack (bonus damage, accrues heat)"
                                </label>
                            </Show>
                            <div class="hint">{hint}</div>
                        }
                    }
                >
                    <div class="winner">{winner_text}</div>
                    <div class="hint">"Reload the page to start a new battle."</div>
                </Show>
            </div>

            <div class="panel">
                <h2>"Roster"</h2>
                <div>{roster}</div>
            </div>

            <div class="panel">
                <h2>"Combat Log"</h2>
                <div class="log">{log}</div>
            </div>
        </div>
    }
}
