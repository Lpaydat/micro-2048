# Deployment Guide

## Quick Start

### Prerequisites
- Domain transferred to Cloudflare
- Hetzner VPS with public IP
- Linera blockchain node running

---

## 🚀 Deployment Steps

### 1. Update Domain Configuration

```bash
# Update all configs with your domain
./scripts/update_domain.sh yourdomain.com
```

### 2. Deploy Frontend to Cloudflare Pages

**Option A: Via Dashboard** (Recommended for first-time setup)

1. Go to [Cloudflare Dashboard](https://dash.cloudflare.com)
2. Navigate to: **Workers & Pages** → **Create application** → **Pages**
3. Connect your Git repository
4. Configure build settings:
   ```
   Build command: cd website && npm install && npm run build
   Build output directory: website/.svelte-kit/cloudflare
   ```
5. Add environment variables (or use defaults from `wrangler.toml`)
6. Deploy!

**Option B: Via CLI**

```bash
cd website
npx wrangler login
npm run deploy
```

### 3. Add Custom Domain in Cloudflare

1. In Cloudflare Pages project → **Custom domains**
2. Add your domain: `yourdomain.com`
3. Cloudflare automatically creates CNAME record

### 4. Configure Backend DNS

In **Cloudflare Dashboard** → **DNS**:

```
Type: A
Name: api
Content: YOUR_HETZNER_SERVER_IP
Proxy: DNS only (gray cloud) ⚠️
```

**Important:** Use "DNS only" (not proxied) for WebSocket support!

### 5. Setup Backend on Hetzner VPS

SSH into your Hetzner server:

```bash
# Clone repository
git clone https://github.com/yourusername/u2048.git
cd u2048

# Install Caddy (if not installed)
sudo pacman -S caddy  # Arch Linux
# OR
sudo apt install caddy  # Ubuntu/Debian

# Start Linera service
linera service --port 8088 &

# Run Caddy setup script
./scripts/setup_caddy.sh yourdomain.com
```

### 6. Verify Deployment

```bash
# Test frontend
curl https://yourdomain.com

# Test backend
curl https://api.yourdomain.com/chains/YOUR_CHAIN_ID/applications/YOUR_APP_ID
```

---

## 📁 Project Structure

```
u2048/
├── scripts/
│   ├── setup_caddy.sh       # Caddy reverse proxy setup
│   └── update_domain.sh     # Update domain in configs
├── website/
│   ├── wrangler.toml        # Cloudflare Pages config
│   └── .env                 # Local development config
└── src/                     # Linera smart contract
```

---

## 🏗️ Architecture

```
┌──────────────────────────────────────────────┐
│         yourdomain.com (Cloudflare DNS)      │
└──────────────────────────────────────────────┘
         │                          │
         │                          │
    ┌────▼────┐              ┌──────▼─────┐
    │ Frontend│              │   Backend  │
    │(CF Pages)│              │  (Hetzner) │
    └─────────┘              └────────────┘
    SvelteKit                Caddy + Linera
```

- **Frontend**: Static site on Cloudflare Pages (free, CDN)
- **Backend**: Linera node on Hetzner VPS with Caddy reverse proxy

---

## 🔧 Configuration Files

### `website/wrangler.toml`
Cloudflare Pages configuration with environment variables for production/preview.

### `scripts/setup_caddy.sh`
Automated Caddy setup:
- Creates reverse proxy config
- Enables automatic HTTPS
- Configures WebSocket support
- Sets up firewall rules

### `scripts/update_domain.sh`
Quick domain update across all config files.

---

## 📝 Useful Commands

### Frontend (Cloudflare Pages)

```bash
cd website

# Local development
npm run dev

# Build
npm run build

# Deploy to Cloudflare
npm run deploy

# Preview deployment
npm run preview
```

### Backend (Hetzner VPS)

```bash
# Check Caddy status
sudo systemctl status caddy

# View Caddy logs
sudo journalctl -u caddy -f

# Reload Caddy config
sudo systemctl reload caddy

# Validate Caddyfile
sudo caddy validate --config /etc/caddy/Caddyfile

# Check Linera service
ps aux | grep linera
```

---

## 🐛 Troubleshooting

### Frontend Issues

**Build fails:**
- Check Cloudflare Pages build logs
- Verify `package.json` scripts
- Ensure all dependencies are in `package.json`

**Domain not working:**
- Wait for DNS propagation (~5-30 min)
- Check CNAME record in Cloudflare DNS
- Verify custom domain is added in Pages project

### Backend Issues

**API not accessible:**
```bash
# Check Linera is running
ps aux | grep linera

# Check Caddy is running
sudo systemctl status caddy

# View Caddy errors
sudo journalctl -u caddy -n 50

# Test Caddy config
sudo caddy validate --config /etc/caddy/Caddyfile
```

**WebSocket not connecting:**
- Ensure DNS record is "DNS only" (gray cloud)
- Check firewall: `sudo ufw status`
- Verify Caddy WebSocket headers in `/etc/caddy/Caddyfile`

**SSL certificate issues:**
```bash
# Caddy logs show certificate errors
sudo journalctl -u caddy -f

# Common fixes:
# 1. Ensure port 80 and 443 are open
# 2. Wait for DNS propagation
# 3. Restart Caddy: sudo systemctl restart caddy
```

---

## 💰 Cost Estimate

- **Cloudflare DNS**: Free
- **Cloudflare Pages**: Free (100,000 requests/day)
- **Hetzner VPS**: €5-20/month
- **SSL Certificates**: Free (Let's Encrypt)
- **Total**: ~€5-20/month

---

## 🔒 Security Checklist

- ✅ HTTPS enforced on frontend and backend
- ✅ Automatic SSL certificate renewal (Caddy)
- ✅ Firewall configured on VPS
- ✅ DNS configured properly
- ✅ No secrets in git repository
- ✅ Environment variables in Cloudflare dashboard

---

## 📚 Additional Resources

- [Cloudflare Pages Docs](https://developers.cloudflare.com/pages/)
- [Caddy Documentation](https://caddyserver.com/docs/)
- [Linera Documentation](https://linera.io/docs)
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/)

---

## 🆘 Getting Help

If you encounter issues:

1. Check the troubleshooting section above
2. Review Caddy logs: `sudo journalctl -u caddy -f`
3. Review Cloudflare Pages build logs
4. Check Linera service logs

For Cloudflare-specific issues, check their [community forum](https://community.cloudflare.com/).
