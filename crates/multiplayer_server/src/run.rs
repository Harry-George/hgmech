use crate::args::Args;
use crate::state::{RequestedRoom, RoomId, ServerState};
use crate::topology::MatchmakingDemoTopology;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use clap::Parser;
use matchbox_signaling::SignalingServerBuilder;
use tracing::info;
use tracing_subscriber::prelude::*;

pub async fn run() {
    fn setup_logging() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    "multiplayer_server=info,tower_http=debug,matchbox_signaling=info".into()
                }),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .compact()
                    .with_file(false)
                    .with_target(false),
            )
            .init();
    }

    setup_logging();
    let args = Args::parse();

    // Setup router
    let host = args.host;
    let ip = if host.ip().is_unspecified() {
        "127.0.0.1".into()
    } else {
        host.ip().to_string()
    };
    info!("Matchbox Signaling Server: {host}");
    info!("Signaling: ws://{ip}:{}/<room_id>", host.port());
    info!("Health check: http://{ip}:{}/health", host.port());
    info!("Rooms: http://{ip}:{}/rooms (GET/POST)", host.port());

    let state = ServerState::default();
    let server = SignalingServerBuilder::new(args.host, MatchmakingDemoTopology, state.clone())
        .on_connection_request({
            let mut state = state.clone();
            #[allow(clippy::result_large_err)]
            move |connection| {
                let room_id = RoomId(connection.path.clone().unwrap_or_default());
                if !state.room_exists(&room_id) {
                    info!("Denying connection to unknown room: {room_id:?}");
                    return Ok(false);
                }
                let room = RequestedRoom { id: room_id };
                state.add_waiting_client(connection.origin, room);
                Ok(true) // allow all clients
            }
        })
        .on_id_assignment({
            let mut state = state.clone();
            move |(origin, peer_id)| {
                info!("Client connected {origin:?}: {peer_id:?}");
                state.assign_id_to_waiting_client(origin, peer_id);
            }
        })
        .cors()
        .trace()
        .mutate_router({
            let state = state.clone();
            move |router| {
                router.route("/health", get(health_handler)).merge(
                    Router::new()
                        .route("/rooms", get(list_rooms_handler).post(create_room_handler))
                        .route("/rooms/{room_id}", get(check_room_handler))
                        .with_state(state),
                )
            }
        })
        .build();
    server
        .serve()
        .await
        .expect("Unable to run signaling server, is it already running?")
}

pub async fn health_handler() -> impl axum::response::IntoResponse {
    axum::http::StatusCode::OK
}

async fn create_room_handler(
    axum::extract::State(state): axum::extract::State<ServerState>,
) -> axum::Json<RoomId> {
    axum::Json(state.create_room())
}

async fn list_rooms_handler(
    axum::extract::State(state): axum::extract::State<ServerState>,
) -> axum::Json<Vec<RoomId>> {
    axum::Json(state.get_rooms())
}

async fn check_room_handler(
    axum::extract::Path(room_id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<ServerState>,
) -> axum::http::StatusCode {
    if state.room_exists(&RoomId(room_id)) {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::NOT_FOUND
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_create_and_check_room() {
        let state = ServerState::default();
        let app = Router::new()
            .route("/rooms", get(list_rooms_handler).post(create_room_handler))
            .route("/rooms/{room_id}", get(check_room_handler))
            .with_state(state);

        // Create room
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/rooms")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let room_id: RoomId = serde_json::from_slice(&body).unwrap();

        // List rooms
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/rooms")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let rooms: Vec<RoomId> = serde_json::from_slice(&body).unwrap();
        assert!(rooms.contains(&room_id));

        // Check room exists
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/rooms/{}", room_id.0))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check non-existent room
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/rooms/non-existent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
