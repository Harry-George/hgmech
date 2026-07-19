# Shared data sources and locals.
#
# The stack runs in the account's default VPC and its (public) default subnets to
# stay simple — Fargate tasks get a public IP so they can pull the image and the
# ALB faces the internet. For an isolated network, swap these data sources for a
# dedicated VPC module.

data "aws_vpc" "default" {
  default = true
}

data "aws_subnets" "default" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.default.id]
  }
}

data "aws_route53_zone" "main" {
  name         = var.domain_name
  private_zone = false
}

data "aws_caller_identity" "current" {}

locals {
  signaling_fqdn = "${var.signaling_subdomain}.${var.domain_name}"
  app_fqdn       = "${var.app_subdomain}.${var.domain_name}"
  image_uri      = "${aws_ecr_repository.server.repository_url}:${var.image_tag}"

  tags = {
    Project   = var.project
    ManagedBy = "terraform"
  }
}
