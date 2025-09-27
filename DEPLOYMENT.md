# Deployment Guide

This guide covers deploying Kiko with SSL/TLS certificates using Docker Compose and Let's Encrypt.

## Prerequisites

- Docker and Docker Compose installed
- Domain name pointing to your server (e.g., `kikopointer.com`)
- Server with ports 80 and 443 accessible
- Email address for Let's Encrypt notifications

## Production Deployment with SSL

### 1. Initial Setup

First, clone the repository and navigate to the project directory:

```bash
git clone <repository-url>
cd kiko
```

### 2. Configure Domain

Update the domain configuration in `scripts/ssl-setup.sh`:

```bash
# Edit these variables if using a different domain
DOMAIN="kikopointer.com"
EMAIL="admin@kikopointer.com"
```

### 3. SSL Certificate Setup

Run the SSL setup script to obtain Let's Encrypt certificates:

```bash
# For production certificates
./scripts/ssl-setup.sh

# For staging/testing certificates (recommended for testing)
./scripts/ssl-setup.sh staging
```

This script will:
1. Start services with HTTP-only nginx configuration
2. Obtain SSL certificates via ACME challenge
3. Switch nginx to SSL configuration
4. Reload nginx with HTTPS enabled

### 4. Verify Deployment

After the script completes, verify your site is accessible:

- **HTTPS**: https://kikopointer.com
- **WWW HTTPS**: https://www.kikopointer.com
- **HTTP Redirect**: http://kikopointer.com (should redirect to HTTPS)

### 5. SSL Certificate Auto-Renewal

Set up automatic certificate renewal using cron:

```bash
# Make the renewal script executable
chmod +x ./scripts/ssl-renew.sh

# Add to crontab (runs weekly on Mondays at 3 AM)
echo "0 3 * * 1 /Users/zack/Workspace/rust/2025/fullstack/kiko/scripts/ssl-renew.sh >> /tmp/ssl-renewal.log 2>&1" | crontab -

# Verify cron job was added
crontab -l
```

## Manual Operations

### Manual Certificate Renewal

To manually renew certificates:

```bash
./scripts/ssl-renew.sh
```

Or run the renewal command directly:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml run --rm certbot renew
docker compose -f docker-compose.yml -f docker-compose.prod.yml exec nginx nginx -s reload
```

### Viewing Certificate Status

Check certificate expiration:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml run --rm certbot certificates
```

### Logs and Troubleshooting

View nginx logs:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml logs nginx
```

View certbot logs:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml logs certbot
```

View SSL renewal logs:

```bash
tail -f /tmp/ssl-renewal.log
```

### Restarting Services

To restart all services:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml down
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

To restart just nginx (after config changes):

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml restart nginx
```

## Architecture Overview

### Production Stack

- **Nginx**: Reverse proxy with SSL termination
- **Frontend**: Yew-based WebAssembly application
- **Backend**: Axum-based REST API and WebSocket server
- **Certbot**: Let's Encrypt certificate management

### SSL Configuration

- **Certificates**: Let's Encrypt with 90-day validity
- **Auto-renewal**: Weekly cron job checks for renewal
- **Security**: Modern TLS 1.2/1.3 configuration with security headers
- **HTTPS Redirect**: All HTTP traffic redirected to HTTPS

### Docker Volumes

- `certbot-etc`: SSL certificates and Let's Encrypt configuration
- `certbot-var`: Let's Encrypt working directory
- `web-root`: ACME challenge files

## Security Features

- **HSTS**: Strict Transport Security enabled
- **CSP**: Content Security Policy configured
- **Rate Limiting**: API and WebSocket endpoints protected
- **Security Headers**: XSS protection, frame options, content type sniffing prevention

## Monitoring

The deployment includes health checks for all services:

- **Backend**: `GET /health`
- **Frontend**: Internal nginx health check
- **Nginx**: Container health monitoring

Check service status:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml ps
```

## Updating the Application

To update to a new version:

1. Pull latest changes:
   ```bash
   git pull origin main
   ```

2. Rebuild and restart services:
   ```bash
   docker compose -f docker-compose.yml -f docker-compose.prod.yml build
   docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
   ```

## Troubleshooting

### Common Issues

1. **Certificate not found**: Ensure SSL setup script completed successfully
2. **Permission denied**: Check that scripts are executable (`chmod +x`)
3. **Domain not resolving**: Verify DNS configuration points to your server
4. **Port conflicts**: Ensure ports 80 and 443 are available

### Emergency HTTP-Only Mode

If SSL is broken and you need HTTP access:

```bash
# Use bootstrap config temporarily
docker compose -f docker-compose.yml -f docker-compose.prod.yml exec nginx \
  cp /etc/nginx/nginx-bootstrap.conf /etc/nginx/nginx.conf
docker compose -f docker-compose.yml -f docker-compose.prod.yml exec nginx nginx -s reload
```

Remember to switch back to SSL config after fixing the issue.