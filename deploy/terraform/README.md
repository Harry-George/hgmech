# AWS deployment (Terraform)

Provisions the `bt` multiplayer **signaling server** on AWS, and (optionally) the
static WASM app.

**What it creates**

- **Signaling server** — ECR repo, ECS Fargate service, Application Load Balancer
  with an ACM certificate (`wss://signal.<domain>`), CloudWatch logs, and a
  Route 53 record. The ALB proxies the WebSocket signaling; game traffic itself is
  peer-to-peer (WebRTC) and never touches AWS.
- **Static app** (when `deploy_static_app = true`) — private S3 bucket behind
  CloudFront with its own ACM cert and Route 53 record (`https://play.<domain>`).

```
 browser ──wss──▶ ALB ──▶ Fargate (signaling)      ← handshake only
 browser ◀─────────  WebRTC data channel  ─────────▶ browser   ← the actual game
```

## Prerequisites

- Terraform ≥ 1.5, the AWS CLI (authenticated), and Docker.
- A domain with an **existing public Route 53 hosted zone** (the config looks it
  up; it does not create it).

## Deploy

```bash
cd deploy/terraform
cp terraform.tfvars.example terraform.tfvars   # set domain_name (at least)
terraform init

# 1) Create the image registry first, so there's somewhere to push to.
terraform apply -target=aws_ecr_repository.server

# 2) Build and push the server image (also rolls the service once it exists).
./push-image.sh

# 3) Create everything else. ACM DNS validation can take a few minutes.
terraform apply
```

The ordering matters only on the first run: the ECS service can't pull an image
that isn't there yet. (If you `terraform apply` everything at once instead, the
service simply retries and goes healthy shortly after step 2.)

Sanity check the server:

```bash
curl "$(terraform output -raw signaling_health_url)"   # -> 200 OK
```

## Build the client against it

```bash
cd ../..                       # repo root
BT_SIGNAL_HTTP=$(terraform -chdir=deploy/terraform output -raw signaling_http_url) \
BT_SIGNAL_WS=$(terraform -chdir=deploy/terraform output -raw signaling_ws_url) \
trunk build --release
```

- **App on GitHub Pages** (`deploy_static_app = false`): publish `dist/` as usual.
- **App on AWS** (`deploy_static_app = true`): upload and invalidate the CDN:

  ```bash
  aws s3 sync dist "s3://$(terraform -chdir=deploy/terraform output -raw app_bucket)/" --delete
  aws cloudfront create-invalidation \
    --distribution-id "$(terraform -chdir=deploy/terraform output -raw app_cloudfront_distribution_id)" \
    --paths '/*'
  ```

  The app is then at `terraform output app_url`.

## Redeploying the server

```bash
./push-image.sh          # rebuild, push, force a new ECS deployment
```

## TURN (optional)

STUN (built into the client) covers most players. If some can't connect (symmetric
NAT / strict firewalls), stand up a TURN relay — e.g. [coturn] on a small EC2
instance, or a hosted provider — and rebuild the client with `BT_TURN_URL`
(+ `BT_TURN_USER` / `BT_TURN_PASS`). This Terraform doesn't provision TURN.

[coturn]: https://github.com/coturn/coturn

## Costs & teardown

Roughly an always-on Fargate task + an ALB (the ALB dominates — a few USD/week).
Scale to zero with `desired_count = 0`, or tear the whole thing down:

```bash
terraform destroy
```
