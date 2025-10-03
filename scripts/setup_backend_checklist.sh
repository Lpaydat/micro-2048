#!/bin/bash

echo "🚀 Backend Setup Checklist for api.micro2048.xyz"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

DOMAIN="micro2048.xyz"

echo "📋 Part 1: Cloudflare DNS Configuration"
echo ""
echo "1. Login to Cloudflare Dashboard:"
echo "   → https://dash.cloudflare.com"
echo ""
echo "2. Navigate to: DNS → Records"
echo ""
echo "3. Add A Record:"
echo "   ${YELLOW}Type:${NC} A"
echo "   ${YELLOW}Name:${NC} api"
echo "   ${YELLOW}IPv4 address:${NC} [YOUR_HETZNER_SERVER_IP]"
echo "   ${YELLOW}Proxy status:${NC} DNS only (gray cloud ☁️) ⚠️ IMPORTANT!"
echo "   ${YELLOW}TTL:${NC} Auto"
echo ""
echo "4. Click Save"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

read -p "Have you added the DNS A record? (y/n): " dns_done
if [ "$dns_done" != "y" ]; then
    echo "${YELLOW}⚠️  Please complete DNS setup first!${NC}"
    exit 1
fi

echo ""
echo "📋 Part 2: Hetzner VPS Setup"
echo ""

# Check if we're on Hetzner VPS
if [ -f /etc/hostname ]; then
    echo "Detected system: $(cat /etc/hostname)"
fi

# Get current IP
echo "Your current IP:"
curl -s ifconfig.me
echo ""
echo ""

# Check Linera
echo "Checking prerequisites..."
echo ""

if command -v linera &> /dev/null; then
    echo "${GREEN}✓${NC} Linera is installed: $(linera --version | head -1)"
else
    echo "${RED}✗${NC} Linera is not installed"
    echo "   Install from: https://linera.io/docs"
fi

# Check Caddy
if command -v caddy &> /dev/null; then
    echo "${GREEN}✓${NC} Caddy is installed: $(caddy version | head -1)"
else
    echo "${RED}✗${NC} Caddy is not installed"
    echo "   Install with:"
    echo "   ${YELLOW}sudo pacman -S caddy${NC}  # Arch Linux"
    echo "   ${YELLOW}sudo apt install caddy${NC}  # Ubuntu/Debian"
    read -p "Install Caddy now? (y/n): " install_caddy
    if [ "$install_caddy" == "y" ]; then
        if command -v pacman &> /dev/null; then
            sudo pacman -S caddy
        elif command -v apt &> /dev/null; then
            sudo apt update && sudo apt install -y caddy
        fi
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if Linera is running
if pgrep -f "linera service" > /dev/null; then
    echo "${GREEN}✓${NC} Linera service is running"
else
    echo "${YELLOW}⚠${NC}  Linera service is not running"
    echo ""
    read -p "Start Linera service on port 8088? (y/n): " start_linera
    if [ "$start_linera" == "y" ]; then
        echo "Starting Linera service..."
        echo "${YELLOW}Note:${NC} Use screen/tmux for persistent service"
        linera service --port 8088 &
        sleep 2
        if pgrep -f "linera service" > /dev/null; then
            echo "${GREEN}✓${NC} Linera service started"
        fi
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Ask to run Caddy setup
read -p "Run Caddy setup script for $DOMAIN? (y/n): " setup_caddy
if [ "$setup_caddy" == "y" ]; then
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [ -f "$SCRIPT_DIR/setup_caddy.sh" ]; then
        echo ""
        echo "Running Caddy setup..."
        bash "$SCRIPT_DIR/setup_caddy.sh" "$DOMAIN"
    else
        echo "${RED}✗${NC} setup_caddy.sh not found!"
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 Part 3: Verification"
echo ""
echo "Test DNS resolution:"
echo "  ${YELLOW}dig api.$DOMAIN${NC}"
echo ""
echo "Test Backend API:"
echo "  ${YELLOW}curl https://api.$DOMAIN${NC}"
echo ""
echo "Check services:"
echo "  ${YELLOW}ps aux | grep linera${NC}"
echo "  ${YELLOW}sudo systemctl status caddy${NC}"
echo ""
echo "View logs:"
echo "  ${YELLOW}sudo journalctl -u caddy -f${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎉 Backend setup complete!"
echo ""
echo "Your backend will be available at:"
echo "  ${GREEN}https://api.micro2048.xyz${NC}"
echo ""
echo "Note: SSL certificate may take a few minutes to be issued."
echo ""
