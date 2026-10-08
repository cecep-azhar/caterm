# CATerm — Product Hunt Launch Kit

## 1. Product Overview & Metadata

- **Product Name:** CATerm
- **Tagline (<= 60 chars):** Ultra-fast native Rust SSH terminal & unified multi-cloud VFS
- **Primary Category:** Developer Tools
- **Secondary Categories:** Productivity, Open Source, Cloud Management
- **Platforms:** Linux (AppImage / Standalone), Windows (.exe / installer), macOS (Apple Silicon & Intel)
- **Website URL:** [https://caterm.fathforce.com](https://caterm.fathforce.com) (or GCC CATerm App)
- **Repository:** [https://github.com/cecep-azhar/caterm](https://github.com/cecep-azhar/caterm)
- **Pricing:** Free forever (Local-first Community) / $6/month Pro / $29-$99 Founder Lifetime Deal

---

## 2. Maker's First Comment

> Hey Product Hunt community! 👋
>
> I'm Cecep Azhar, founder of Fathforce. As engineers managing dozens of cloud instances, bare-metal nodes, and edge micro-clusters every single day, we got fed up with two painful extremes:
>
> 1. Bloated Electron terminal clients that consume 400MB+ RAM before you even type `ssh`, stutter under heavy scrollback, and lock core SFTP behind steep recurring subscriptions.
> 2. Fragmented tools where you need PuTTY/Termius for SSH, Cyberduck/WinSCP for SFTP/S3, and ad-hoc scripts for server benchmarking.
>
> That's why we engineered **CATerm** from the ground up in native **Rust**:
>
> - 🚀 **Ultra-fast cold start (<100ms) & 17MB native binary footprint:** Powered by Rust's `portable-pty` and Tokio async I/O. Near-zero input latency and ~30MB idle RAM.
> - 🛡️ **Zero-Knowledge Local-First Architecture:** Master credentials, private keys, and session history stay on your local disk, encrypted via SQLCipher (AES-256) + Argon2id. Zero telemetry, zero tracking, zero cloud login lock-in.
> - 🌐 **Unified VFS Storage Explorer:** True dual-pane file management across SFTP, Amazon S3, MinIO, Cloudflare R2, WebDAV, and FTP in one unified interface with transfer resume and background queueing.
> - ⚡ **DevOps & SRE Diagnostics Lab:** Native, zero-dependency server diagnostics. Run Internet speedtests, QoS bufferbloat grading, Unixbench/sysbench scoring, and Mesh Matrix inter-node latency checks directly from your workstation without installing Python or external runtimes on remote servers.
> - 👥 **Pro for Teams & P2P Envelope Sync:** 7-seat team sharing with zero-knowledge cryptographic envelopes (DEK/KEK architecture). Sync host entries and credentials peer-to-peer without intermediate servers reading your keys.
>
> The desktop core is 100% free and local. For Product Hunt launch day, we're offering **50% OFF** Pro subscriptions with code **`PH50`**, plus a strictly limited **$99 Founder Lifetime Deal** (first 100 seats).
>
> Would love your honest feedback, critique, and feature requests. What does your current server management setup look like?

---

## 3. 60-Second Video Demo Script

| Timestamp | Visual / Screen Action | Voiceover / Sound (Audio) |
|---|---|---|
| **00:00 - 00:08** | Side-by-side terminal comparison. Heavy Electron tool takes 4s to launch; CATerm launches instantly (<100ms). System monitor shows 17MB binary and 30MB idle RAM. | "Still waiting for your Electron terminal to launch? Meet CATerm: a native Rust terminal that boots in under 100 milliseconds." |
| **00:08 - 00:20** | User opens SSH session with split pane. Typing is instantaneous with zero stutter. Embedded hardware bar shows CPU/RAM telemetry without server agents. | "Built on portable-pty and Tokio async I/O, CATerm brings zero-delay typing even over intercontinental links—plus real-time agentless host telemetry." |
| **00:20 - 00:32** | Open dual-pane VFS explorer. Left: Local disk. Right: Switching seamlessly between an AWS S3 bucket, Cloudflare R2, and remote SFTP. File dragged across panes. | "No more juggling WinSCP or Cyberduck. CATerm's unified VFS lets you drag and drop files across SFTP, S3, MinIO, R2, and WebDAV in a single dual-pane view." |
| **00:32 - 00:44** | Click on 'DevOps Lab'. Trigger Internet Speedtest and Mesh Matrix. Matrix displays colored latency cells between 5 servers in Frankfurt, Singapore, and New York. | "Need to troubleshoot? CATerm's built-in SRE lab tests QoS bufferbloat and inter-node mesh latency without installing Python or scripts on remote hosts." |
| **00:44 - 00:52** | Settings screen showing SQLCipher AES-256 local vault status, RFC 6238 TOTP auto-fill, and P2P encrypted team envelope sync. | "Your secrets stay yours. Local-first SQLCipher AES-256 encryption, zero tracking, and zero-knowledge team sync." |
| **00:52 - 01:00** | End card with CATerm logo, website caterm.fathforce.com, GitHub badge, and code 'PH50'. | "Download CATerm free today. Use code PH50 for launch perks. Take back control of your terminal." |

---

## 4. Product Hunt Launch Day Checklist

### Pre-Launch (T-24h to T-1h)
- [ ] **Launch Timing:** Schedule post for exactly **00:01 AM PST** (Product Hunt day cycle start).
- [ ] **Assets Checklist:**
  - Thumbnail GIF/PNG (240x240 px, high contrast Rust logo).
  - Gallery Images (1270x760 px, min. 5 slides: Hero, VFS Dual-Pane, SRE Diagnostics Lab, Comparison Matrix, Zero-Knowledge Security).
  - YouTube/Vimeo demo video link (60-sec demo script above).
- [ ] **Promo Setup:**
  - Lemon Squeezy / Stripe coupon code `PH50` activated (50% recurring or first-year discount).
  - Limited slot setup: $99 Founder Lifetime deal (cap at 100 licenses).
  - IDR QRIS payment link tested via Midtrans / Mayar.
- [ ] **Landing Page Parity:** Monorepo GCC landing page updated with killer features and comparison matrix; `templ generate` tested and deployed.

### Launch Day Execution (T+0h to T+24h)
- [ ] **00:01 PST:** Publish listing and immediately post Maker's First Comment.
- [ ] **Internal Community Mobilization:** Share launch announcement in internal Slack, Discord, Telegram, and developer circles (strictly asking for authentic feedback and discussion, no bot upvotes to avoid shadowban).
- [ ] **Social Media Blitz:**
  - Launch thread on X/Twitter with tag `#ProductHunt #RustLang #DevOps #SysAdmin`.
  - LinkedIn post sharing the story of moving from Electron to native Rust.
  - Subreddit sharing on r/rust, r/selfhosted, r/devops (focusing on technical open-core architecture, no hard sales).
- [ ] **Real-time Monitoring & Reply:** Answer every single comment on Product Hunt within 15 minutes.
- [ ] **T+12h Update:** Share milestone update in comments (e.g. "#1 Product of the Day contender, 500 downloads reached").

---

## 5. Payment Gateway Configuration & Operational Guide

CATerm supports both global and Indonesian regional payment pathways to maximize checkout conversion.

### A. Global Payments (Lemon Squeezy / Stripe)
- **Merchant of Record (MoR):** Lemon Squeezy handles global VAT/sales tax, invoices, and card processing.
- **Product Tiers:**
  1. *CATerm Pro Monthly:* $6/month (Promo code `PH50` -> $3/month).
  2. *CATerm Pro Yearly:* $60/year (Promo code `PH50` -> $30/year).
  3. *Founder Lifetime Tier:* $99 one-time (First 100 Product Hunt buyers).
- **Webhook Integration:**
  - Webhook URL: `https://caterm.fathforce.com/api/webhooks/lemonsqueezy`
  - Events: `order_created`, `subscription_created`, `subscription_cancelled`, `subscription_updated`.
  - Action: Validates HMAC SHA-256 signature, generates offline-verifiable Ed25519 license key, sends license email to customer.

### B. Indonesian Local Payments (Midtrans / Mayar — IDR QRIS & Virtual Accounts)
- **Use Case:** Local developers, students, and Indonesian tech firms requiring QRIS, BCA/Mandiri/BRI Virtual Account, or GoPay/OVO.
- **Pricing Parity:**
  - Founder Lifetime: Rp 1.499.000 (diskon promo PH ~Rp 990.000).
  - Pro Monthly: Rp 89.000/bulan.
- **Checkout Flow:**
  1. User selects "Bayar via QRIS / VA (Indonesia)" on landing page.
  2. Mayar / Midtrans Snap popup initializes invoice.
  3. Webhook listener `https://caterm.fathforce.com/api/webhooks/midtrans` or `/mayar` receives payment confirmation.
  4. Instant license key dispatch via email and WhatsApp gateway.
