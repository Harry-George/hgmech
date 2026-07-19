//! Optional peer-to-peer multiplayer over [matchbox].
//!
//! Two browsers connect through the project's matchbox **signaling server**
//! (`crates/multiplayer_server`), which brokers a direct WebRTC data channel
//! between them. Once connected they play a shared game by **full-state
//! snapshotting**: after every committed local action the acting client
//! serializes the entire [`GameState`] to JSON and sends it to its peer, which
//! replaces its own copy. Because the turn machine only lets the *current*
//! player act (and each client is pinned to one player index), exactly one side
//! ever mutates the game at a time, so last-write-wins is always correct and
//! there is nothing to merge.
//!
//! Everything that actually touches the network is gated to `wasm32`. Native
//! builds (`cargo test` / `cargo check`) compile the same API as no-ops so the
//! rest of the UI crate keeps building off the browser.
//!
//! [matchbox]: https://github.com/johanhelsing/matchbox

use leptos::prelude::*;

use super::{Game, Screen};

#[cfg(target_arch = "wasm32")]
use super::ScreenKind;
#[cfg(target_arch = "wasm32")]
use game::dice::XorShiftDice;
#[cfg(target_arch = "wasm32")]
use game::state::GameState;
#[cfg(target_arch = "wasm32")]
use matchbox_socket::{PeerState, WebRtcSocket};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

/// The single reliable data channel opened by [`WebRtcSocket::new_reliable`].
#[cfg(target_arch = "wasm32")]
const CHANNEL_ID: usize = 0;

/// Base URL of the signaling server's HTTP API (room creation / lookup).
///
/// Points at a locally-running `multiplayer_server` by default; change these two
/// constants (or serve the server behind the same origin) for a deployment.
pub const SIGNAL_HTTP: &str = "http://127.0.0.1:3536";
/// Base URL of the signaling server's WebSocket endpoint.
pub const SIGNAL_WS: &str = "ws://127.0.0.1:3536";

/// How the local player is participating in the current game.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    /// Hot-seat / single-machine play — no networking (the original behaviour).
    Local,
    /// Created the room; plays player 0 and sets up the match.
    Host,
    /// Joined an existing room by id; plays player 1.
    Join,
}

/// Connection lifecycle, surfaced to the lobby UI.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnStatus {
    /// Not connecting (Local play, or nothing started yet).
    Idle,
    /// Host is asking the server to create a room.
    Creating,
    /// Socket is open; waiting for the opponent to appear.
    Waiting,
    /// The peer has connected.
    Connected,
    /// The peer dropped after having connected.
    Disconnected,
    /// Something went wrong (see the browser console for details).
    Error,
}

/// Shared multiplayer state, placed in context by `App`. All fields are cheap
/// reactive/arena handles so `Net` is `Copy` and can be captured freely by event
/// handlers and the poll timer.
#[derive(Clone, Copy)]
pub struct Net {
    /// Current participation mode.
    pub role: RwSignal<Role>,
    /// Which player index this client controls (0 = host, 1 = joiner).
    pub my_player: RwSignal<usize>,
    /// The room id — shown to the host so the opponent can join.
    pub room_id: RwSignal<String>,
    /// Connection lifecycle for the lobby UI.
    pub status: RwSignal<ConnStatus>,
    /// The live WebRTC socket, held in non-`Send` local storage (web-sys types
    /// are not thread-safe; the whole app is single-threaded in the browser).
    #[cfg(target_arch = "wasm32")]
    socket: StoredValue<Option<WebRtcSocket>, LocalStorage>,
}

impl Net {
    /// Create a fresh Local (offline) networking context.
    pub fn new() -> Self {
        Self {
            role: RwSignal::new(Role::Local),
            my_player: RwSignal::new(0),
            room_id: RwSignal::new(String::new()),
            status: RwSignal::new(ConnStatus::Idle),
            #[cfg(target_arch = "wasm32")]
            socket: StoredValue::new_local(None),
        }
    }

    /// Whether the local client is allowed to act while `current_player` is the
    /// active player. Always true in Local play; online, only when it is this
    /// client's own player's turn (so neither side can drive the other's units).
    pub fn my_turn(&self, current_player: usize) -> bool {
        self.role.get_untracked() == Role::Local
            || self.my_player.get_untracked() == current_player
    }

    /// True when networked (Host or Join) rather than Local.
    pub fn is_online(&self) -> bool {
        self.role.get_untracked() != Role::Local
    }

    /// Start hosting: create a room on the signaling server, then open the
    /// socket and wait for an opponent. The room id is published on `room_id`
    /// for display as soon as the server returns it.
    #[allow(unused_variables)]
    pub fn host(&self) {
        self.role.set(Role::Host);
        self.my_player.set(0);
        self.status.set(ConnStatus::Creating);
        self.room_id.set(String::new());

        #[cfg(target_arch = "wasm32")]
        {
            let net = *self;
            spawn_local(async move {
                match create_room().await {
                    Ok(room) => {
                        net.room_id.set(room.clone());
                        net.open_socket(&room);
                        net.status.set(ConnStatus::Waiting);
                    }
                    Err(e) => {
                        tracing::error!("failed to create room: {e}");
                        net.status.set(ConnStatus::Error);
                    }
                }
            });
        }
    }

    /// Join an existing room by id and wait for the connection to establish.
    #[allow(unused_variables)]
    pub fn join(&self, room: String) {
        self.role.set(Role::Join);
        self.my_player.set(1);
        self.room_id.set(room.clone());
        self.status.set(ConnStatus::Waiting);

        #[cfg(target_arch = "wasm32")]
        {
            self.open_socket(&room);
        }
    }

    /// Open a reliable WebRTC socket to `room` and spawn its message loop.
    #[cfg(target_arch = "wasm32")]
    fn open_socket(&self, room: &str) {
        let url = format!("{SIGNAL_WS}/{room}");
        let (socket, loop_fut) = WebRtcSocket::new_reliable(url);
        self.socket.set_value(Some(socket));
        // The loop future drives the underlying signaling/WebRTC machinery; it
        // must be polled for any message to flow. It resolves when the socket
        // closes, which we treat as the end of the session.
        spawn_local(async move {
            let _ = loop_fut.await;
        });
    }

    /// Send the full current game state to the connected peer. No-op in Local
    /// play or before a peer has connected. Call this right after any committed
    /// local mutation of the game.
    #[allow(unused_variables)]
    pub fn broadcast(&self, game: Game) {
        if self.role.get_untracked() == Role::Local {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let Some(json) = game.with_untracked(|g| serde_json::to_string(g).ok())
            else {
                tracing::error!("failed to serialize game state");
                return;
            };
            let packet: Box<[u8]> = json.into_bytes().into_boxed_slice();
            self.socket.update_value(|slot| {
                let Some(socket) = slot.as_mut() else { return };
                let peers: Vec<_> = socket.connected_peers().collect();
                for peer in peers {
                    socket.channel_mut(CHANNEL_ID).send(packet.clone(), peer);
                }
            });
        }
    }

    /// Pump the socket: pick up peer connect/disconnect events and any inbound
    /// state snapshots, applying them to `game`/`screen`. Driven by a short
    /// interval timer from `App`. No-op until a socket exists.
    #[allow(unused_variables)]
    pub fn poll(&self, game: Game, screen: Screen) {
        #[cfg(target_arch = "wasm32")]
        {
            let mut peer_events: Vec<PeerState> = Vec::new();
            let mut packets: Vec<Box<[u8]>> = Vec::new();

            self.socket.update_value(|slot| {
                let Some(socket) = slot.as_mut() else { return };
                for (_peer, state) in socket.update_peers() {
                    peer_events.push(state);
                }
                for (_peer, packet) in socket.channel_mut(CHANNEL_ID).receive() {
                    packets.push(packet);
                }
            });

            for state in peer_events {
                match state {
                    PeerState::Connected => {
                        self.status.set(ConnStatus::Connected);
                        // The host now sets up the match (force selection); the
                        // joiner waits for the host's opening snapshot.
                        if self.role.get_untracked() == Role::Host {
                            screen.0.set(ScreenKind::ForceSelect);
                        }
                    }
                    PeerState::Disconnected => {
                        self.status.set(ConnStatus::Disconnected);
                    }
                }
            }

            for packet in packets {
                let Ok(text) = std::str::from_utf8(&packet) else {
                    continue;
                };
                match serde_json::from_str::<GameState<XorShiftDice>>(text) {
                    Ok(new_state) => {
                        game.set(new_state);
                        // The first snapshot the joiner receives is its cue to
                        // enter the battle; later ones just refresh the board.
                        if screen.0.get_untracked() != ScreenKind::Battle {
                            screen.0.set(ScreenKind::Battle);
                        }
                    }
                    Err(e) => tracing::error!("failed to parse peer state: {e}"),
                }
            }
        }
    }
}

impl Default for Net {
    fn default() -> Self {
        Self::new()
    }
}

/// Ask the signaling server to create a fresh room, returning its id.
///
/// The server only accepts WebSocket connections to rooms it has minted, so the
/// host must create one over HTTP before opening its socket.
#[cfg(target_arch = "wasm32")]
async fn create_room() -> Result<String, String> {
    let resp = gloo_net::http::Request::post(&format!("{SIGNAL_HTTP}/rooms"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("server returned {}", resp.status()));
    }
    // `RoomId` is a newtype over `String`, so it serializes as a bare JSON string.
    resp.json::<String>().await.map_err(|e| e.to_string())
}

/// Read the multiplayer context provided by `App`.
pub fn use_net() -> Net {
    expect_context::<Net>()
}
