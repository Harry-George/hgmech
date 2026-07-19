//! The full Alpha Strike "card" for a single unit, shown as a modal overlay when
//! the player clicks a unit's name. Reads the shared game state reactively so the
//! current-vs-printed stats (armor/structure/heat/criticals) stay live while the
//! card is open.

use leptos::prelude::*;

use super::{use_game, use_selected_card};

#[component]
pub fn UnitCard() -> impl IntoView {
    let game = use_game();
    let selected = use_selected_card();

    let close = move |_| selected.set(None);

    // The whole card body, recomputed reactively from the game state. Returns
    // `None` when nothing is selected (or the id has somehow gone away), which
    // Leptos renders as empty.
    let body = move || {
        let id = selected.get()?;
        game.with(|g| {
            let u = g.unit(id)?;
            let card = &u.card;

            // Prefer the master-list artwork; fall back to a generated avatar.
            let img_src = if card.image_url.is_empty() {
                format!(
                    "https://api.dicebear.com/7.x/bottts/svg?seed={}&backgroundColor={}",
                    card.name, u.color
                )
            } else {
                card.image_url.clone()
            };

            // Special abilities, verbatim from the master list (many are not
            // modelled by the engine, so show the raw text).
            let specials = if card.specials.is_empty() {
                "—".to_string()
            } else {
                card.specials.clone()
            };

            let armor_pct = u.cur_armor as f64 / card.armor.max(1) as f64 * 100.0;
            let structure_pct = u.cur_structure as f64 / card.structure.max(1) as f64 * 100.0;

            // Show live degraded values next to the printed ones when they differ.
            let mv_line = if (u.cur_mv - card.mv_inches).abs() > f64::EPSILON {
                format!("{:.0}\" (of {:.0}\")", u.cur_mv, card.mv_inches)
            } else {
                format!("{:.0}\"", card.mv_inches)
            };
            let dmg = |cur: u32, base: u32| {
                if cur != base {
                    format!("{cur} ({base})")
                } else {
                    format!("{base}")
                }
            };
            let dmg_s = dmg(u.cur_damage.short, card.damage.short);
            let dmg_m = dmg(u.cur_damage.medium, card.damage.medium);
            let dmg_l = dmg(u.cur_damage.long, card.damage.long);

            let title = format!("{} — {} (Size {})", card.name, card.size.as_str(), card.size.value());
            let role = if card.role.is_empty() { "—" } else { card.role.as_str() };
            let subtitle = format!("Player {} • {} • PV {}", u.player, role, card.pv);

            let status = if u.is_destroyed() {
                Some(("DESTROYED", "badge--shutdown"))
            } else if u.shutdown {
                Some(("SHUTDOWN", "badge--shutdown"))
            } else {
                None
            };

            Some(
                view! {
                    <div class="card__header">
                        <img class="card__avatar" src=img_src alt=card.name.clone() />
                        <div class="card__titles">
                            <div class="card__name">{title}</div>
                            <div class="card__sub">
                                {subtitle}
                                {status.map(|(label, cls)| view! {
                                    <span class=format!("badge {cls} card__status")>{label}</span>
                                })}
                            </div>
                        </div>
                    </div>

                    <div class="card__grid">
                        <div class="card__stat"><span>"Move (MV)"</span><b>{mv_line}</b></div>
                        <div class="card__stat"><span>"TMM"</span><b>{format!("{}", u.cur_tmm)}</b></div>
                        <div class="card__stat"><span>"Pilot Skill"</span><b>{format!("{}", card.pilot_skill)}</b></div>
                        <div class="card__stat"><span>"Overheat (OV)"</span><b>{format!("{}", card.overheat)}</b></div>
                    </div>

                    <div class="card__damage">
                        <div class="card__stat"><span>"Short"</span><b>{dmg_s}</b></div>
                        <div class="card__stat"><span>"Medium"</span><b>{dmg_m}</b></div>
                        <div class="card__stat"><span>"Long"</span><b>{dmg_l}</b></div>
                    </div>

                    <div class="card__bars">
                        <div class="card__bar-row">
                            <span class="card__bar-label">"Armor"</span>
                            <div class="bar bar--wide">
                                <div class="bar__fill bar__fill--armor" style=format!("width:{armor_pct}%;")></div>
                            </div>
                            <span class="card__bar-val">{format!("{}/{}", u.cur_armor, card.armor)}</span>
                        </div>
                        <div class="card__bar-row">
                            <span class="card__bar-label">"Structure"</span>
                            <div class="bar bar--wide">
                                <div class="bar__fill bar__fill--structure" style=format!("width:{structure_pct}%;")></div>
                            </div>
                            <span class="card__bar-val">{format!("{}/{}", u.cur_structure, card.structure)}</span>
                        </div>
                    </div>

                    <div class="card__specials">
                        <span>"Specials"</span>
                        <b>{specials}</b>
                    </div>

                    <div class="card__grid">
                        <div class="card__stat"><span>"Heat"</span><b>{format!("{}", u.heat)}</b></div>
                        <div class="card__stat"><span>"Move mode"</span><b>{u.mode.as_str()}</b></div>
                        <div class="card__stat"><span>"Engine hits"</span><b>{format!("{}", u.engine_hits)}</b></div>
                        <div class="card__stat"><span>"Fire-control hits"</span><b>{format!("{}", u.fire_control_hits)}</b></div>
                    </div>
                }
                .into_any(),
            )
        })
    };

    view! {
        <Show when=move || selected.get().is_some()>
            <div class="card-modal" on:click=close>
                <div class="card" on:click=|e| e.stop_propagation()>
                    <button class="card__close" on:click=close>"×"</button>
                    {body}
                </div>
            </div>
        </Show>
    }
}
