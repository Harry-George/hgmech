//! SVG overlay drawn across the whole board: movement paths with distance
//! labels (Movement phase) and range rings for the selected attacker (Attack
//! phase). Purely presentational and non-interactive (`pointer-events: none`).

use leptos::prelude::*;

use game::combat::{LONG_RANGE, MEDIUM_RANGE, SHORT_RANGE};
use game::scenario::{BOARD_H, BOARD_W};
use game::state::Phase;

use super::{use_game, SCALE};

#[component]
pub fn Overlays() -> impl IntoView {
    let game = use_game();
    let width = BOARD_W * SCALE;
    let height = BOARD_H * SCALE;

    // Home-edge deployment zones, highlighted for the player placing now.
    let zones = move || {
        game.with(|g| {
            if g.phase != Phase::Deployment {
                return Vec::new();
            }
            let active = g.current_player();
            (0..g.num_players)
                .map(|p| {
                    let (x0, x1, y0, y1) = g.deployment_zone(p);
                    let color = if p == 0 { "#ff5a5a" } else { "#5aa0ff" };
                    let (fill_op, sw) = if p == active { ("0.16", "2") } else { ("0.05", "1") };
                    view! {
                        <rect
                            x=(x0 * SCALE).to_string()
                            y=(y0 * SCALE).to_string()
                            width=((x1 - x0) * SCALE).to_string()
                            height=((y1 - y0) * SCALE).to_string()
                            fill=color
                            fill-opacity=fill_op
                            stroke=color
                            stroke-width=sw
                            stroke-dasharray="6,5"
                        />
                    }
                })
                .collect::<Vec<_>>()
        })
    };

    // Movement paths for every unit with a committed/in-progress move.
    let movement = move || {
        game.with(|g| {
            if g.phase != Phase::Movement {
                return Vec::new();
            }
            let mut out = Vec::new();
            for u in &g.units {
                if u.is_destroyed() {
                    continue;
                }
                let over_budget = u.available_movement() < 0.5;
                let color = if over_budget { "#ff5a5a" } else { "#6ee06e" };
                for line in u.get_lines() {
                    let len = line.length();
                    if len < 0.01 {
                        continue;
                    }
                    let (x1, y1) = line.start;
                    let (x2, y2) = line.end;
                    let mx = (x1 + x2) / 2.0 * SCALE;
                    let my = (y1 + y2) / 2.0 * SCALE;
                    out.push(view! {
                        <g>
                            <line
                                x1=(x1 * SCALE).to_string()
                                y1=(y1 * SCALE).to_string()
                                x2=(x2 * SCALE).to_string()
                                y2=(y2 * SCALE).to_string()
                                stroke=color
                                stroke-width="2"
                            />
                            <circle cx=(x1 * SCALE).to_string() cy=(y1 * SCALE).to_string() r="4" fill=color />
                            <text
                                x=mx.to_string()
                                y=my.to_string()
                                fill=color
                                font-size="13"
                                font-weight="bold"
                                text-anchor="middle"
                                style="paint-order:stroke;stroke:#000;stroke-width:3px;"
                            >
                                {format!("{len:.0}\"")}
                            </text>
                        </g>
                    });
                }
            }
            out
        })
    };

    // Range rings centred on the selected attacker.
    let rings = move || {
        game.with(|g| {
            if g.phase != Phase::Attack {
                return Vec::new();
            }
            let Some(att) = g.selected_attacker.and_then(|id| g.unit(id)) else {
                return Vec::new();
            };
            let (cx, cy) = att.position;
            let cx = cx * SCALE;
            let cy = cy * SCALE;
            [
                (SHORT_RANGE, "#6ee06e", "S"),
                (MEDIUM_RANGE, "#ffd23f", "M"),
                (LONG_RANGE, "#ff8a5a", "L"),
            ]
            .into_iter()
            .map(|(r, color, label)| {
                let radius = r * SCALE;
                view! {
                    <g>
                        <circle
                            cx=cx.to_string()
                            cy=cy.to_string()
                            r=radius.to_string()
                            fill="none"
                            stroke=color
                            stroke-width="1.5"
                            stroke-dasharray="6,5"
                            opacity="0.7"
                        />
                        <text
                            x=cx.to_string()
                            y=(cy - radius + 12.0).to_string()
                            fill=color
                            font-size="12"
                            text-anchor="middle"
                            style="paint-order:stroke;stroke:#000;stroke-width:3px;"
                        >
                            {label}
                        </text>
                    </g>
                }
            })
            .collect::<Vec<_>>()
        })
    };

    view! {
        <svg class="overlay" width=width.to_string() height=height.to_string()>
            {zones}
            {movement}
            {rings}
        </svg>
    }
}
