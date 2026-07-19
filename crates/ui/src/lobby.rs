//! Pre-game connection screens: the three-way **Host / Join / Local** chooser
//! and the lobby that follows an online choice (the host's room id while it
//! waits, or the joiner's room-id entry box).

use leptos::prelude::*;

use super::net::{use_net, ConnStatus, Role};
use super::{use_screen, ScreenKind};

/// The opening screen: pick how to play.
#[component]
pub fn ModeSelect() -> impl IntoView {
    let net = use_net();
    let screen = use_screen();

    let play_local = move |_| {
        net.role.set(Role::Local);
        net.status.set(ConnStatus::Idle);
        screen.set(ScreenKind::ForceSelect);
    };
    let host = move |_| {
        net.host();
        screen.set(ScreenKind::Lobby);
    };
    let join = move |_| {
        // Switch to the lobby's room-id entry step; the socket opens on submit.
        net.role.set(Role::Join);
        net.my_player.set(1);
        net.status.set(ConnStatus::Idle);
        net.room_id.set(String::new());
        screen.set(ScreenKind::Lobby);
    };

    view! {
        <div class="lobby">
            <div class="lobby__card">
                <h1>"Alpha Strike"</h1>
                <p class="hint">"Choose how you want to play."</p>
                <div class="lobby__modes">
                    <button class="btn lobby__mode" on:click=host>
                        <span class="lobby__mode-title">"Host"</span>
                        <span class="lobby__mode-sub">"Create an online game and invite an opponent"</span>
                    </button>
                    <button class="btn lobby__mode" on:click=join>
                        <span class="lobby__mode-title">"Join"</span>
                        <span class="lobby__mode-sub">"Enter a room id to join a friend's game"</span>
                    </button>
                    <button class="btn lobby__mode" on:click=play_local>
                        <span class="lobby__mode-title">"Local"</span>
                        <span class="lobby__mode-sub">"Hot-seat on this device — both sides, one screen"</span>
                    </button>
                </div>
            </div>
        </div>
    }
}

/// The waiting room shown after choosing Host or Join.
#[component]
pub fn Lobby() -> impl IntoView {
    let net = use_net();
    let screen = use_screen();

    let room_entry = RwSignal::new(String::new());

    let back = move |_| {
        net.role.set(Role::Local);
        net.status.set(ConnStatus::Idle);
        screen.set(ScreenKind::ModeSelect);
    };

    let connect = move |_| {
        let id = room_entry.get().trim().to_string();
        if !id.is_empty() {
            net.join(id);
        }
    };

    let is_host = move || net.role.get() == Role::Host;
    let status = move || net.status.get();

    view! {
        <div class="lobby">
            <div class="lobby__card">
                <Show
                    when=is_host
                    fallback=move || {
                        // ---- Joiner ----
                        view! {
                            <h1>"Join a Game"</h1>
                            <Show
                                when=move || status() == ConnStatus::Idle
                                fallback=move || {
                                    view! {
                                        <p class="lobby__status">{move || join_status_text(status())}</p>
                                    }
                                }
                            >
                                <p class="hint">"Paste the room id your opponent gave you."</p>
                                <input
                                    class="lobby__input"
                                    type="text"
                                    placeholder="room id…"
                                    prop:value=move || room_entry.get()
                                    on:input=move |e| room_entry.set(event_target_value(&e))
                                />
                                <button
                                    class="btn"
                                    prop:disabled=move || room_entry.get().trim().is_empty()
                                    on:click=connect
                                >
                                    "Connect"
                                </button>
                            </Show>
                        }
                    }
                >
                    // ---- Host ----
                    <h1>"Hosting a Game"</h1>
                    <Show
                        when=move || net.status.get() == ConnStatus::Waiting
                        fallback=move || {
                            view! {
                                <p class="lobby__status">{move || host_status_text(net.status.get())}</p>
                            }
                        }
                    >
                        <p class="hint">"Share this room id with your opponent. The game starts when they join."</p>
                        <div class="lobby__room">{move || net.room_id.get()}</div>
                        <p class="lobby__status">"Waiting for opponent to join…"</p>
                    </Show>
                </Show>

                <button class="lobby__back" on:click=back>"← Back"</button>
            </div>
        </div>
    }
}

/// Human-readable label for the host's connection status.
fn host_status_text(status: ConnStatus) -> &'static str {
    match status {
        ConnStatus::Creating => "Creating room…",
        ConnStatus::Waiting => "Waiting for opponent to join…",
        ConnStatus::Connected => "Opponent connected — starting…",
        ConnStatus::Disconnected => "Opponent disconnected.",
        ConnStatus::Error => "Could not reach the game server.",
        ConnStatus::Idle => "",
    }
}

/// Human-readable label for the joiner's connection status.
fn join_status_text(status: ConnStatus) -> &'static str {
    match status {
        ConnStatus::Waiting => "Connecting…",
        ConnStatus::Connected => "Connected — waiting for the host to set up the battle…",
        ConnStatus::Disconnected => "Host disconnected.",
        ConnStatus::Error => "Could not reach the game server.",
        ConnStatus::Creating | ConnStatus::Idle => "",
    }
}
