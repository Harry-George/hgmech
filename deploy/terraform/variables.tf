variable "region" {
  description = "AWS region for the signaling server and its load balancer."
  type        = string
  default     = "eu-west-2"
}

variable "project" {
  description = "Name prefix for all resources (kept short — used in ALB/target-group names, which cap at 32 chars)."
  type        = string
  default     = "bt-multiplayer"
}

variable "domain_name" {
  description = "A domain you own with an existing public Route 53 hosted zone (e.g. \"example.com\"). Certificates and DNS records are created under it."
  type        = string
}

variable "signaling_subdomain" {
  description = "Subdomain for the signaling server. The client should be built with BT_SIGNAL_WS=wss://<this>.<domain_name>."
  type        = string
  default     = "signal"
}

variable "image_tag" {
  description = "Tag of the server image in the created ECR repository to run."
  type        = string
  default     = "latest"
}

variable "desired_count" {
  description = "Number of Fargate tasks to run. 1 is fine for a hobby game; raise for redundancy."
  type        = number
  default     = 1
}

variable "task_cpu" {
  description = "Fargate task CPU units (256 = 0.25 vCPU). String, per the ECS API."
  type        = string
  default     = "256"
}

variable "task_memory" {
  description = "Fargate task memory in MiB. String, per the ECS API."
  type        = string
  default     = "512"
}

variable "container_port" {
  description = "Port the signaling server listens on inside the container (matches HOST in the image)."
  type        = number
  default     = 3536
}

variable "log_retention_days" {
  description = "CloudWatch log retention for the server."
  type        = number
  default     = 14
}

# ---- Optional static-app hosting (S3 + CloudFront) -------------------------
# The game's WASM bundle is already served from GitHub Pages, so this is off by
# default. Enable it to host the built `dist/` on AWS instead.

variable "deploy_static_app" {
  description = "Also provision S3 + CloudFront to host the built WASM app."
  type        = bool
  default     = true
}

variable "app_subdomain" {
  description = "Subdomain for the static app when deploy_static_app is true."
  type        = string
  default     = "play"
}
