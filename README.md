# HGMECH

This is a hobby project to be able to play aplhastrike in a browser.

It is primarily written to learn web technologies.

The live site: <https://harry-george.github.io/hgmech/>

## Multiplayer

The game can be played **locally** (hot-seat, both sides on one screen) or
**online** against another browser. When you start a game you pick one of three
options:

- **Host** — creates an online room and shows a room id; share it with your
  opponent. Play begins when they join. The host is Player 0 and builds both
  forces during setup.
- **Join** — paste a host's room id to connect as Player 1.
- **Local** — the original hot-seat experience, no networking.

### How it works

Online play uses [matchbox](https://github.com/johanhelsing/matchbox):

- A small **signaling server** (`crates/multiplayer_server`) brokers the initial
  handshake over WebSocket + HTTP. It only pairs peers by room id — no game logic
  runs on it.
- After the handshake the two browsers connect **directly** (WebRTC data
  channel). The game stays in sync by shipping a full serialized `GameState`
  snapshot to the peer after every committed action; since only the active
  player can act, last-write-wins is always correct.
- **STUN** (built in) handles NAT traversal for most home networks. Restrictive
  or symmetric NATs additionally need a **TURN** relay (see the env vars below).

### Running it locally

```bash
./dev.sh
```

This starts the signaling server on `:3536` and `trunk serve` on `:8080`. Open
two browser tabs at <http://127.0.0.1:8080>: click **Host** in one, then **Join**
in the other with the room id it shows.

### Configuring the signaling endpoint

The client's server URLs are baked in at **build time** from environment
variables (they default to `127.0.0.1:3536` for local dev):

| Variable         | Purpose                       | Example                      |
|------------------|-------------------------------|------------------------------|
| `BT_SIGNAL_HTTP` | HTTP base for room creation   | `https://signal.example.com` |
| `BT_SIGNAL_WS`   | WebSocket base for signaling  | `wss://signal.example.com`   |
| `BT_TURN_URL`    | Optional TURN relay           | `turn:turn.example.com:3478` |
| `BT_TURN_USER`   | TURN username (if required)   | `user`                       |
| `BT_TURN_PASS`   | TURN credential (if required) | `pass`                       |

Use `https`/`wss` in production — a page served over HTTPS may not open plain
`http`/`ws` connections. Build the deployed bundle like:

```bash
BT_SIGNAL_HTTP=https://signal.example.com \
BT_SIGNAL_WS=wss://signal.example.com \
trunk build --release
```

### Deploying the server

A container image and a Terraform config for AWS (ECS Fargate + ALB + ACM +
Route 53, with optional S3/CloudFront static hosting) live under
[`deploy/terraform/`](deploy/terraform/) — see its README for the full workflow.
Build the image with the repo-root Dockerfile:

```bash
docker build -f crates/multiplayer_server/Dockerfile -t bt-multiplayer .
```
