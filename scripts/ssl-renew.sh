#!/bin/bash

# SSL Certificate Renewal Script for Kiko
# Add this to crontab to run weekly: 0 3 * * 1 /Users/zack/Workspace/rust/2025/fullstack/kiko/scripts/ssl-renew.sh

set -e

# Change to project directory
cd /Users/zack/Workspace/rust/2025/fullstack/kiko

echo "🔄 $(date): Checking SSL certificate renewal for kikopointer.com"

# Try to renew certificates
docker compose -f docker-compose.yml -f docker-compose.prod.yml run --rm certbot renew --quiet

# Reload nginx to pick up any renewed certificates
docker compose -f docker-compose.yml -f docker-compose.prod.yml exec nginx nginx -s reload

echo "✅ $(date): Certificate renewal check completed"