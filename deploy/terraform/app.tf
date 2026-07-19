# Optional static hosting for the built WASM app: a private S3 bucket fronted by
# CloudFront (HTTPS, custom domain). Gated on `deploy_static_app`.
#
# Upload the built `dist/` to the bucket after `apply` — see the README.

locals {
  app_count      = var.deploy_static_app ? 1 : 0
  app_bucket_name = "${var.project}-app-${data.aws_caller_identity.current.account_id}"
}

# ---- Certificate (must be in us-east-1 for CloudFront) ---------------------

resource "aws_acm_certificate" "app" {
  count             = local.app_count
  provider          = aws.us_east_1
  domain_name       = local.app_fqdn
  validation_method = "DNS"

  lifecycle {
    create_before_destroy = true
  }

  tags = local.tags
}

resource "aws_route53_record" "app_cert_validation" {
  for_each = var.deploy_static_app ? {
    for dvo in aws_acm_certificate.app[0].domain_validation_options : dvo.domain_name => {
      name   = dvo.resource_record_name
      type   = dvo.resource_record_type
      record = dvo.resource_record_value
    }
  } : {}

  zone_id         = data.aws_route53_zone.main.zone_id
  name            = each.value.name
  type            = each.value.type
  records         = [each.value.record]
  ttl             = 60
  allow_overwrite = true
}

resource "aws_acm_certificate_validation" "app" {
  count                   = local.app_count
  provider                = aws.us_east_1
  certificate_arn         = aws_acm_certificate.app[0].arn
  validation_record_fqdns = [for r in aws_route53_record.app_cert_validation : r.fqdn]
}

# ---- Bucket (private; reached only through CloudFront) ---------------------

resource "aws_s3_bucket" "app" {
  count         = local.app_count
  bucket        = local.app_bucket_name
  force_destroy = true
  tags          = local.tags
}

resource "aws_s3_bucket_public_access_block" "app" {
  count                   = local.app_count
  bucket                  = aws_s3_bucket.app[0].id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# ---- CloudFront ------------------------------------------------------------

resource "aws_cloudfront_origin_access_control" "app" {
  count                             = local.app_count
  name                              = "${var.project}-app"
  origin_access_control_origin_type = "s3"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

# AWS-managed caching policy tuned for static content.
data "aws_cloudfront_cache_policy" "optimized" {
  name = "Managed-CachingOptimized"
}

resource "aws_cloudfront_distribution" "app" {
  count               = local.app_count
  enabled             = true
  is_ipv6_enabled     = true
  default_root_object = "index.html"
  aliases             = [local.app_fqdn]
  comment             = "${var.project} static app"
  price_class         = "PriceClass_100"

  origin {
    domain_name              = aws_s3_bucket.app[0].bucket_regional_domain_name
    origin_id                = "s3-app"
    origin_access_control_id = aws_cloudfront_origin_access_control.app[0].id
  }

  default_cache_behavior {
    target_origin_id       = "s3-app"
    viewer_protocol_policy  = "redirect-to-https"
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    cache_policy_id        = data.aws_cloudfront_cache_policy.optimized.id
    compress               = true
  }

  # Single-page app: serve index.html for unknown paths instead of S3's XML error.
  custom_error_response {
    error_code            = 403
    response_code         = 200
    response_page_path    = "/index.html"
    error_caching_min_ttl = 10
  }
  custom_error_response {
    error_code            = 404
    response_code         = 200
    response_page_path    = "/index.html"
    error_caching_min_ttl = 10
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    acm_certificate_arn      = aws_acm_certificate_validation.app[0].certificate_arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.2_2021"
  }

  tags = local.tags
}

# Allow only this CloudFront distribution to read the bucket.
data "aws_iam_policy_document" "app_bucket" {
  count = local.app_count

  statement {
    actions   = ["s3:GetObject"]
    resources = ["${aws_s3_bucket.app[0].arn}/*"]

    principals {
      type        = "Service"
      identifiers = ["cloudfront.amazonaws.com"]
    }

    condition {
      test     = "StringEquals"
      variable = "AWS:SourceArn"
      values   = [aws_cloudfront_distribution.app[0].arn]
    }
  }
}

resource "aws_s3_bucket_policy" "app" {
  count  = local.app_count
  bucket = aws_s3_bucket.app[0].id
  policy = data.aws_iam_policy_document.app_bucket[0].json
}

# ---- DNS -------------------------------------------------------------------

resource "aws_route53_record" "app" {
  count   = local.app_count
  zone_id = data.aws_route53_zone.main.zone_id
  name    = local.app_fqdn
  type    = "A"

  alias {
    name                   = aws_cloudfront_distribution.app[0].domain_name
    zone_id                = aws_cloudfront_distribution.app[0].hosted_zone_id
    evaluate_target_health = false
  }
}
