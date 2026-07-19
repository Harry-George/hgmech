//! Pre-game force-selection screen. Both players build a force at the same time
//! by picking BattleMechs from the master unit list. There is no PV budget — the
//! screen just tallies each side's unit count and total Point Value. Each pick's
//! pilot skill is adjustable. "Start Battle" builds the game and moves on to the
//! on-board Deployment phase.

use leptos::prelude::*;

use game::dice::XorShiftDice;
use game::scenario::{demo_terrain, BOARD_H, BOARD_W};
use game::state::{GameState, DEPLOY_DEPTH};
use game::unit::UnitCard as UnitCardData;
use game::unit::UnitState;

use super::{use_catalog, use_forces, use_game, use_screen, Forces, Pick, ScreenKind};

/// Weight-class options for the size filter dropdown.
const SIZES: [&str; 4] = ["Light", "Medium", "Heavy", "Assault"];

/// Maximum catalog rows rendered at once (the list is filtered/searched down to
/// this; a note prompts the user to refine when there are more matches).
const ROW_CAP: usize = 150;

/// A stable-ish colour for player `player`'s `index`-th unit, so avatars differ.
fn pick_color(player: usize, index: usize) -> String {
    const P0: [&str; 6] = ["FF5A5A", "E0743A", "D94F70", "C0392B", "E67E22", "B0506A"];
    const P1: [&str; 6] = ["5AA0FF", "6D5AE0", "3AB0C0", "2E86DE", "8E44AD", "4C7FAF"];
    let palette = if player == 0 { P0 } else { P1 };
    palette[index % palette.len()].to_string()
}

/// Turn the two chosen forces into a game ready for deployment. Each unit is
/// seeded at a spot inside its owner's home-edge zone, spaced vertically.
fn build_game(forces: &Forces, seed: u64) -> GameState<XorShiftDice> {
    let mut units = Vec::new();
    let mut id = 0usize;
    for player in 0..2 {
        let picks = &forces.players[player];
        let n = picks.len().max(1);
        let x = if player == 0 {
            DEPLOY_DEPTH / 2.0
        } else {
            BOARD_W - DEPLOY_DEPTH / 2.0
        };
        for (i, pick) in picks.iter().enumerate() {
            let mut card = pick.card.clone();
            card.pilot_skill = pick.pilot_skill;
            let y = BOARD_H * (i as f64 + 1.0) / (n as f64 + 1.0);
            units.push(UnitState::new(id, player, card, pick.color.clone(), (x, y)));
            id += 1;
        }
    }
    GameState::new_deploying(units, demo_terrain(), XorShiftDice::new(seed))
}

#[component]
pub fn ForceSelect() -> impl IntoView {
    let catalog = use_catalog();
    let forces = use_forces();
    let game = use_game();
    let screen = use_screen();

    let search = RwSignal::new(String::new());
    let role_filter = RwSignal::new(String::new());
    let size_filter = RwSignal::new(String::new());

    // Distinct roles for the filter dropdown (computed once).
    let roles = catalog.with_value(|cat| {
        let mut r: Vec<String> = cat
            .iter()
            .map(|u| u.role.clone())
            .filter(|s| !s.is_empty())
            .collect();
        r.sort();
        r.dedup();
        r
    });

    // Indices into the catalog that match the current search/filters (one past
    // the cap is kept so the "refine" hint knows the list was truncated).
    let filtered = Memo::new(move |_| {
        let q = search.get().to_lowercase();
        let role = role_filter.get();
        let size = size_filter.get();
        catalog.with_value(|cat| {
            cat.iter()
                .enumerate()
                .filter(|(_, u)| {
                    (q.is_empty() || u.name.to_lowercase().contains(&q))
                        && (role.is_empty() || u.role == role)
                        && (size.is_empty() || u.size.as_str() == size)
                })
                .take(ROW_CAP + 1)
                .map(|(i, _)| i)
                .collect::<Vec<usize>>()
        })
    });

    let add_unit = move |player: usize, card: UnitCardData| {
        forces.update(|f| {
            let index = f.players[player].len();
            let color = pick_color(player, index);
            f.players[player].push(Pick {
                card,
                pilot_skill: 4,
                color,
            });
        });
    };

    let rows = move || {
        let idxs = filtered.get();
        idxs.iter()
            .take(ROW_CAP)
            .map(|&i| {
                let (name, meta) = catalog.with_value(|cat| {
                    let u = &cat[i];
                    (
                        u.name.clone(),
                        format!(
                            "PV {} • {} • {:.0}\" • {}/{}/{} • {}",
                            u.pv,
                            u.size.as_str(),
                            u.mv_inches,
                            u.damage.short,
                            u.damage.medium,
                            u.damage.long,
                            if u.role.is_empty() { "—" } else { &u.role },
                        ),
                    )
                });
                let add0 = move |_| add_unit(0, catalog.with_value(|c| c[i].clone()));
                let add1 = move |_| add_unit(1, catalog.with_value(|c| c[i].clone()));
                view! {
                    <div class="fs__row">
                        <div class="fs__row-info">
                            <span class="fs__row-name">{name}</span>
                            <span class="fs__row-meta">{meta}</span>
                        </div>
                        <div class="fs__row-add">
                            <button class="btn btn--p0" on:click=add0>"+ P0"</button>
                            <button class="btn btn--p1" on:click=add1>"+ P1"</button>
                        </div>
                    </div>
                }
            })
            .collect_view()
    };

    // One player's chosen roster (reactive).
    let roster = move |player: usize| {
        forces.with(|f| {
            f.players[player]
                .iter()
                .enumerate()
                .map(|(i, pick)| {
                    let name = pick.card.name.clone();
                    let pv = pick.card.pv;
                    let skill = pick.pilot_skill;
                    let dec = move |_| {
                        forces.update(|f| {
                            if let Some(p) = f.players[player].get_mut(i) {
                                p.pilot_skill = (p.pilot_skill - 1).max(0);
                            }
                        });
                    };
                    let inc = move |_| {
                        forces.update(|f| {
                            if let Some(p) = f.players[player].get_mut(i) {
                                p.pilot_skill = (p.pilot_skill + 1).min(6);
                            }
                        });
                    };
                    let remove = move |_| {
                        forces.update(|f| {
                            if i < f.players[player].len() {
                                f.players[player].remove(i);
                            }
                        });
                    };
                    view! {
                        <div class="fs__pick">
                            <span class="fs__pick-name">{name}</span>
                            <span class="fs__pick-pv">{format!("PV {pv}")}</span>
                            <span class="fs__skill">
                                <button class="fs__step" on:click=dec>"−"</button>
                                <span class="fs__skill-val" title="Pilot skill">{format!("PS {skill}")}</span>
                                <button class="fs__step" on:click=inc>"+"</button>
                            </span>
                            <button class="fs__remove" title="Remove" on:click=remove>"✕"</button>
                        </div>
                    }
                })
                .collect_view()
        })
    };

    let totals = move |player: usize| {
        forces.with(|f| {
            let picks = &f.players[player];
            let pv: u32 = picks.iter().map(|p| p.card.pv).sum();
            format!("{} units • {} PV", picks.len(), pv)
        })
    };

    let can_start =
        move || forces.with(|f| !f.players[0].is_empty() && !f.players[1].is_empty());

    let start = move |_| {
        if !can_start() {
            return;
        }
        let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
        let new_game = forces.with(|f| build_game(f, seed));
        game.set(new_game);
        screen.set(ScreenKind::Battle);
    };

    view! {
        <div class="force-select">
            <div class="fs__head">
                <h1>"Build Your Forces"</h1>
                <p class="hint">
                    "Both players pick BattleMechs from the master list — no point limit, just build what you like. Set each pilot's skill, then start the battle to deploy."
                </p>
            </div>

            <div class="fs__rosters">
                <div class="fs__roster fs__roster--p0">
                    <div class="fs__roster-head">
                        <h2>"Player 0"</h2>
                        <span class="fs__total">{move || totals(0)}</span>
                    </div>
                    <div class="fs__picks">{move || roster(0)}</div>
                </div>
                <div class="fs__roster fs__roster--p1">
                    <div class="fs__roster-head">
                        <h2>"Player 1"</h2>
                        <span class="fs__total">{move || totals(1)}</span>
                    </div>
                    <div class="fs__picks">{move || roster(1)}</div>
                </div>
            </div>

            <div class="fs__catalog panel">
                <div class="fs__filters">
                    <input
                        class="fs__search"
                        type="text"
                        placeholder="Search by name…"
                        prop:value=move || search.get()
                        on:input=move |e| search.set(event_target_value(&e))
                    />
                    <select on:change=move |e| role_filter.set(event_target_value(&e))>
                        <option value="">"All roles"</option>
                        {roles
                            .into_iter()
                            .map(|r| view! { <option value=r.clone()>{r.clone()}</option> })
                            .collect_view()}
                    </select>
                    <select on:change=move |e| size_filter.set(event_target_value(&e))>
                        <option value="">"All sizes"</option>
                        {SIZES
                            .iter()
                            .map(|s| view! { <option value=*s>{*s}</option> })
                            .collect_view()}
                    </select>
                </div>
                <div class="fs__list">{rows}</div>
                <Show when=move || { filtered.get().len() > ROW_CAP }>
                    <div class="hint">
                        {format!("Showing the first {ROW_CAP} matches — refine your search to see more.")}
                    </div>
                </Show>
            </div>

            <button class="btn fs__start" prop:disabled=move || !can_start() on:click=start>
                "Start Battle"
            </button>
        </div>
    }
}
