# Point the signaling subdomain at the load balancer.

resource "aws_route53_record" "signaling" {
  zone_id = data.aws_route53_zone.main.zone_id
  name    = local.signaling_fqdn
  type    = "A"

  alias {
    name                   = aws_lb.signaling.dns_name
    zone_id                = aws_lb.signaling.zone_id
    evaluate_target_health = true
  }
}
