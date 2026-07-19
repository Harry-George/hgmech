output "region" {
  description = "Region the stack is deployed in."
  value       = var.region
}

output "ecr_repository_url" {
  description = "Push the server image here (see push-image.sh)."
  value       = aws_ecr_repository.server.repository_url
}

output "ecs_cluster" {
  description = "ECS cluster name (for force-new-deployment on redeploy)."
  value       = aws_ecs_cluster.main.name
}

output "ecs_service" {
  description = "ECS service name (for force-new-deployment on redeploy)."
  value       = aws_ecs_service.server.name
}

output "signaling_ws_url" {
  description = "Build the client with BT_SIGNAL_WS set to this."
  value       = "wss://${local.signaling_fqdn}"
}

output "signaling_http_url" {
  description = "Build the client with BT_SIGNAL_HTTP set to this."
  value       = "https://${local.signaling_fqdn}"
}

output "signaling_health_url" {
  description = "Sanity check once the service is running."
  value       = "https://${local.signaling_fqdn}/health"
}

output "alb_dns_name" {
  description = "Raw load balancer hostname (behind the Route 53 record)."
  value       = aws_lb.signaling.dns_name
}

output "app_url" {
  description = "Public URL of the static app, when deploy_static_app is true."
  value       = var.deploy_static_app ? "https://${local.app_fqdn}" : null
}

output "app_bucket" {
  description = "S3 bucket to upload the built dist/ into, when hosting the app."
  value       = var.deploy_static_app ? aws_s3_bucket.app[0].bucket : null
}

output "app_cloudfront_distribution_id" {
  description = "CloudFront distribution id (for cache invalidation on redeploy)."
  value       = var.deploy_static_app ? aws_cloudfront_distribution.app[0].id : null
}
