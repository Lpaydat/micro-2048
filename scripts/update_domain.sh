#!/bin/bash

set -e

DOMAIN="${1}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
WRANGLER_FILE="$PROJECT_DIR/website/wrangler.toml"

if [ -z "$DOMAIN" ]; then
    echo "❌ Error: Domain name is required"
    echo "Usage: $0 <domain>"
    echo "Example: $0 u2048.com"
    exit 1
fi

echo "🔧 Updating domain to: $DOMAIN"

# Update wrangler.toml production config
if [ -f "$WRANGLER_FILE" ]; then
    # Update PUBLIC_WEBSITE in production vars
    sed -i.bak "s|PUBLIC_WEBSITE = \"api\.[^\"]*\"|PUBLIC_WEBSITE = \"api.$DOMAIN\"|g" "$WRANGLER_FILE"
    
    # Update default vars if exists
    sed -i.bak "s|PUBLIC_WEBSITE = \"[^\"]*\"|PUBLIC_WEBSITE = \"$DOMAIN\"|" "$WRANGLER_FILE" || true
    
    echo "✅ Updated wrangler.toml"
    echo ""
    echo "📋 Configuration:"
    echo "   Frontend: https://$DOMAIN"
    echo "   Backend API: https://api.$DOMAIN"
    echo ""
    echo "🚀 Next steps:"
    echo "   1. Deploy frontend to Cloudflare Pages:"
    echo "      cd website && npm run deploy"
    echo ""
    echo "   2. Configure DNS in Cloudflare Dashboard:"
    echo "      - CNAME: $DOMAIN → your-project.pages.dev (automatic)"
    echo "      - A record: api.$DOMAIN → YOUR_HETZNER_IP (DNS only, gray cloud)"
    echo ""
    echo "   3. Setup backend on Hetzner VPS:"
    echo "      ./scripts/setup_caddy.sh $DOMAIN"
else
    echo "❌ Error: wrangler.toml not found at $WRANGLER_FILE"
    exit 1
fi
