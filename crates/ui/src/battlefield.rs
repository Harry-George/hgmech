//! The board: a fixed-size, inch-scaled container holding the terrain, the SVG
//! overlay, and one [`UnitView`] per unit.

use leptos::prelude::*;

use game::scenario::{BOARD_H, BOARD_W};
use game::terrain::TerrainKind;

use super::overlays::Overlays;
use super::unit_view::UnitView;
use super::{use_game, SCALE};

#[component]
pub fn Battlefield() -> impl IntoView {
    let game = use_game();

    // Units and terrain are fixed for the match — read them once.
    let ids = game.with_untracked(|g| g.units.iter().map(|u| u.id).collect::<Vec<_>>());
    let terrain = game.with_untracked(|g| g.terrain.clone());

    let width = BOARD_W * SCALE;
    let height = BOARD_H * SCALE;

    let terrain_views = terrain
        .into_iter()
        .map(|t| {
            let class = match t.kind {
                TerrainKind::Woods => "terrain terrain--woods",
                TerrainKind::Cover => "terrain terrain--cover",
                TerrainKind::Open => "terrain",
            };
            let style = format!(
                "left:{}px; top:{}px; width:{}px; height:{}px;",
                t.x * SCALE,
                t.y * SCALE,
                t.w * SCALE,
                t.h * SCALE
            );
            view! { <div class=class style=style>{t.label}</div> }
        })
        .collect_view();

    let unit_views = ids
        .into_iter()
        .map(|id| view! { <UnitView id=id /> })
        .collect_view();

    let board_style = format!("width:{width}px; height:{height}px;");
    let grid_style = format!("background-size:{}px {}px;", SCALE * 2.0, SCALE * 2.0);

    view! {
        <div id="bt-board" class="board" style=board_style>
            <div class="board__grid" style=grid_style></div>
            {terrain_views}
            <Overlays />
            {unit_views}
        </div>
    }
}
