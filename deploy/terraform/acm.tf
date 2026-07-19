# TLS certificate for the signaling endpoint, DNS-validated through Route 53.
# Lives in the stack's region because it terminates on the ALB.

resource "aws_acm_certificate" "signaling" {
  domain_name       = local.signaling_fqdn
  validation_method = "DNS"

  lifecycle {
    create_before_destroy = true
  }

  tags = local.tags
}

resource "aws_route53_record" "signaling_cert_validation" {
  for_each = {
    for dvo in aws_acm_certificate.signaling.domain_validation_options : dvo.domain_name => {
      name   = dvo.resource_record_name
      type   = dvo.resource_record_type
      record = dvo.resource_record_value
    }
  }

  zone_id         = data.aws_route53_zone.main.zone_id
  name            = each.value.name
  type            = each.value.type
  records         = [each.value.record]
  ttl             = 60
  allow_overwrite = true
}

resource "aws_acm_certificate_validation" "signaling" {
  certificate_arn         = aws_acm_certificate.signaling.arn
  validation_record_fqdns = [for r in aws_route53_record.signaling_cert_validation : r.fqdn]
}
