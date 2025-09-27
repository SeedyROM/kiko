#!/bin/bash

# SSL Setup Script for Kiko
# This script sets up SSL certificates using Let's Encrypt

set -e

DOMAIN="kikopointer.com"
EMAIL="admin@kikopointer.com"
STAGING=${1:-false}

echo "🔒 Setting up SSL for ${DOMAIN}"

# Create necessary directories
mkdir -p ./data/certbot/conf
mkdir -p ./data/certbot/www

# Check if running in staging mode
if [ "$STAGING" = "staging" ]; then
    echo "⚠️  Running in STAGING mode (test certificates)"
    CERTBOT_ARGS="--staging"
else
    echo "✅ Running in PRODUCTION mode (real certificates)"
    CERTBOT_ARGS=""
fi

echo "📋 Step 1: Starting services with bootstrap nginx configuration"
# Start all services with bootstrap nginx (HTTP only for ACME challenge)
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

echo "⏳ Waiting for services to start..."
sleep 15

echo "📋 Step 2: Obtaining SSL certificate"
# Run certbot to obtain certificate
docker compose -f docker-compose.yml -f docker-compose.prod.yml run --rm certbot \
    certonly --webroot \
    --webroot-path=/var/www/html \
    --email $EMAIL \
    --agree-tos \
    --no-eff-email \
    $CERTBOT_ARGS \
    -d $DOMAIN \
    -d www.$DOMAIN

echo "📋 Step 3: Switching to SSL nginx configuration"
# Switch to SSL configuration
docker compose -f docker-compose.yml -f docker-compose.prod.yml exec nginx \
    cp /etc/nginx/nginx-ssl.conf /etc/nginx/nginx.conf

echo "📋 Step 4: Reloading nginx with SSL configuration"
# Reload nginx with SSL configuration
docker compose -f docker-compose.yml -f docker-compose.prod.yml exec nginx nginx -s reload

echo "✅ SSL setup complete!"
echo ""
echo "🌐 Your site should now be available at:"
echo "   https://${DOMAIN}"
echo "   https://www.${DOMAIN}"
echo ""
echo "🔄 Certificate will auto-renew every 80 days"
echo ""
echo "📝 To renew manually:"
echo "   docker compose -f docker-compose.yml -f docker-compose.prod.yml run --rm certbot renew"