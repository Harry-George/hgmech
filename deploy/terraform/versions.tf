terraform {
  required_version = ">= 1.5"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.40"
    }
  }
}

provider "aws" {
  region = var.region
}

# CloudFront's ACM certificate must live in us-east-1 regardless of the region
# the rest of the stack runs in. Only used when `deploy_static_app = true`.
provider "aws" {
  alias  = "us_east_1"
  region = "us-east-1"
}
