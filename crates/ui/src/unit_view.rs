//! A single unit on the board: avatar, name, Armor/Structure bars, status
//! badges, drag-to-move (Movement phase) and click-to-target (Attack phase).
//!
//! Dragging uses Pointer Events (`pointerdown`/`pointermove`/`pointerup`) so a
//! single code path covers mouse, touch, and pen. `setPointerCapture` routes
//! every move/up event for the gesture back to the avatar even when the
//! finger/cursor leaves it, replacing the old document-level mouse listeners.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::PointerEvent;

use game::state::Phase;

use super::{use_game, use_overheat, use_selected_card, SCALE};

/// Convert a viewport pixel coordinate to board inches using the live position
/// of the `#bt-board` element (robust to scrolling and layout changes).
fn client_to_board_inches(client_x: f64, client_y: f64) -> Option<(f64, f64)> {
    let board = web_sys::window()?.document()?.get_element_by_id("bt-board")?;
    let rect = board.get_bounding_client_rect();
    Some((
        (client_x - rect.left()) / SCALE,
        (client_y - rect.top()) / SCALE,
    ))
}

#[component]
pub fn UnitView(id: usize) -> impl IntoView {
    let game = use_game();
    let overheat = use_overheat();
    let selected_card = use_selected_card();
    let is_dragging = RwSignal::new(false);

    // Immutable identity/stat fields — read once.
    let (player, color, name, image_url, max_armor, max_structure) = game.with_untracked(|g| {
        let u = g.unit(id).expect("unit id must exist");
        (
            u.player,
            u.color.clone(),
            u.card.name.clone(),
            u.card.image_url.clone(),
            u.card.armor.max(1),
            u.card.structure.max(1),
        )
    });
    // Prefer the master-list artwork; fall back to a generated avatar.
    let img_src = if image_url.is_empty() {
        format!("https://api.dicebear.com/7.x/bottts/svg?seed={name}&backgroundColor={color}")
    } else {
        image_url
    };

    let on_pointerdown = move |e: PointerEvent| {
        let can_drag = game.with(|g| {
            matches!(g.phase, Phase::Movement | Phase::Deployment) && g.is_actionable(id)
        });
        if !can_drag {
            // Leave the event alone so the Attack-phase tap (on:click) and any
            // synthesized touch behaviour still work.
            return;
        }
        // Stop the browser turning the gesture into scroll/zoom/text-selection.
        e.prevent_default();
        // Route all subsequent pointermove/pointerup events for this pointer to
        // the avatar, even if the finger/cursor leaves it mid-drag.
        if let Some(target) = e.target() {
            if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                let _ = el.set_pointer_capture(e.pointer_id());
            }
        }
        // Lock this activation to the unit being dragged so the per-unit
        // alternation tracks the right unit (Movement §2 Step 2, or the
        // matching Deployment placement alternation).
        game.update(|g| match g.phase {
            Phase::Deployment => g.selected_deployer = Some(id),
            _ => g.selected_mover = Some(id),
        });
        is_dragging.set(true);
    };

    let on_pointermove = move |e: PointerEvent| {
        if !is_dragging.get_untracked() {
            return;
        }
        if let Some(target) = client_to_board_inches(e.client_x() as f64, e.client_y() as f64) {
            game.update(|g| match g.phase {
                // Deployment placement clamps to the home zone; ordinary
                // movement clamps to the MV budget.
                Phase::Deployment => g.set_deploy_position(id, target),
                _ => {
                    if let Some(u) = g.unit_mut(id) {
                        u.set_position(target);
                    }
                }
            });
        }
    };

    // Both a normal release and a cancelled gesture (e.g. the OS steals the
    // touch) end the drag identically.
    let end_drag = move || {
        if !is_dragging.get_untracked() {
            return;
        }
        is_dragging.set(false);
        game.update(|g| {
            // Deployment writes position directly (already clamped); only
            // movement commits a path segment.
            if g.phase == Phase::Movement {
                if let Some(u) = g.unit_mut(id) {
                    u.finalise_position();
                }
            }
        });
    };
    let on_pointerup = move |_e: PointerEvent| end_drag();
    let on_pointercancel = move |_e: PointerEvent| end_drag();

    // Attack-phase click: select one of your units, then click an enemy to fire.
    let on_click = move |_| {
        game.update(|g| {
            if g.phase != Phase::Attack {
                return;
            }
            if g.is_actionable(id) {
                g.selected_attacker = Some(id);
                return;
            }
            if let Some(attacker) = g.selected_attacker {
                let attacker_player = g.unit(attacker).map(|u| u.player);
                let my_player = g.unit(id).map(|u| u.player);
                let dead = g.unit(id).map(|u| u.is_destroyed()).unwrap_or(true);
                if attacker_player.is_some() && attacker_player != my_player && !dead {
                    g.attack(attacker, id, overheat.get_untracked());
                }
            }
        });
    };

    let on_undo = move |_| {
        // Go through the engine so a full undo releases the activation lock and
        // lets the player pick a different unit to move.
        game.update(|g| g.undo_unit_move(id));
    };

    // Reactive views ----------------------------------------------------
    let style = move || {
        let (x, y) = game.with(|g| g.unit(id).map(|u| u.position).unwrap_or((0.0, 0.0)));
        let cursor = if is_dragging.get() { "grabbing" } else { "grab" };
        format!("left:{}px; top:{}px; cursor:{cursor};", x * SCALE, y * SCALE)
    };

    let class = move || {
        let mut c = String::from("unit");
        c.push_str(if player == 0 { " unit--p0" } else { " unit--p1" });
        game.with(|g| {
            if g.unit(id).map(|u| u.is_destroyed()).unwrap_or(false) {
                c.push_str(" unit--dead");
                return;
            }
            let selected = g.selected_attacker == Some(id);
            if selected {
                c.push_str(" unit--selected");
            } else if g.is_actionable(id) {
                c.push_str(" unit--active");
            }
            if g.phase == Phase::Attack {
                if let Some(att) = g.selected_attacker {
                    let att_player = g.unit(att).map(|u| u.player);
                    if att_player.is_some() && att_player != Some(player) && g.is_actionable(att) {
                        c.push_str(" unit--targetable");
                    }
                }
            }
        });
        c
    };

    let armor_pct =
        move || game.with(|g| g.unit(id).map(|u| u.cur_armor).unwrap_or(0)) as f64 / max_armor as f64
            * 100.0;
    let structure_pct = move || {
        game.with(|g| g.unit(id).map(|u| u.cur_structure).unwrap_or(0)) as f64
            / max_structure as f64
            * 100.0
    };

    let show_undo = move || {
        game.with(|g| {
            g.phase == Phase::Movement
                && g.is_actionable(id)
                && g.unit(id).map(|u| !u.lines.is_empty()).unwrap_or(false)
        })
    };
    let shutdown = move || game.with(|g| g.unit(id).map(|u| u.shutdown).unwrap_or(false));
    let heat = move || game.with(|g| g.unit(id).map(|u| u.heat).unwrap_or(0));

    view! {
        <div class=class style=style>
            <img
                class="unit__avatar"
                src=img_src
                alt=name.clone()
                width="48"
                height="48"
                on:pointerdown=on_pointerdown
                on:pointermove=on_pointermove
                on:pointerup=on_pointerup
                on:pointercancel=on_pointercancel
                on:click=on_click
            />
            <div class="bar">
                <div class="bar__fill bar__fill--armor" style=move || format!("width:{}%;", armor_pct())></div>
            </div>
            <div class="bar">
                <div class="bar__fill bar__fill--structure" style=move || format!("width:{}%;", structure_pct())></div>
            </div>
            <div
                class="unit__name unit__name--link"
                title="Show unit card"
                on:click=move |_| selected_card.set(Some(id))
            >{name.clone()}</div>
            <Show when=shutdown>
                <span class="badge badge--shutdown">"SHUTDOWN"</span>
            </Show>
            <Show when=move || { heat() > 0 && !shutdown() }>
                <span class="badge" style="background:#7a3b10;color:#ffd">{move || format!("heat {}", heat())}</span>
            </Show>
            <Show when=show_undo>
                <button class="unit__btn" on:click=on_undo>"Undo"</button>
            </Show>
        </div>
    }
}
