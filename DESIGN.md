---
version: alpha
name: CADS - CATerm Design System
description: Cyberpunk monoline wireframe and tech-brutalist HUD design system for sovereign terminal developer tooling.
colors:
  primary: "#EDEDED"
  secondary: "#A1A1AA"
  tertiary: "#06B6D4"
  neutral: "#18181B"
  surface: "#0A0A0C"
  surface-elevated: "#121217"
  surface-subtle: "#18181F"
  border: "#272732"
  brand: "#06B6D4"
  danger: "#F43F5E"
  success: "#10B981"
  warning: "#F59E0B"
typography:
  h1:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 1.5rem
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "-0.02em"
  h2:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 1.25rem
    fontWeight: 700
    lineHeight: 1.3
    letterSpacing: "-0.01em"
  h3:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 0.875rem
    fontWeight: 700
    lineHeight: 1.4
  body-md:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 0.75rem
    fontWeight: 400
    lineHeight: 1.5
  mono-sm:
    fontFamily: "JetBrains Mono, ui-monospace, monospace"
    fontSize: 0.6875rem
    fontWeight: 500
    lineHeight: 1.4
rounded:
  sm: 6px
  md: 8px
  lg: 12px
  xl: 16px
spacing:
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 32px
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "#0A0A0C"
    rounded: "{rounded.md}"
    padding: 8px
  button-primary-hover:
    backgroundColor: "#D4D4D8"
    textColor: "#0A0A0C"
    rounded: "{rounded.md}"
    padding: 8px
  button-secondary:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.primary}"
    rounded: "{rounded.md}"
    padding: 8px
  button-secondary-hover:
    backgroundColor: "#27272A"
    textColor: "{colors.primary}"
    rounded: "{rounded.md}"
    padding: 8px
  button-outline:
    backgroundColor: "transparent"
    textColor: "{colors.primary}"
    rounded: "{rounded.md}"
    padding: 8px
  button-danger:
    backgroundColor: "rgba(244,63,94,0.1)"
    textColor: "{colors.danger}"
    rounded: "{rounded.md}"
    padding: 8px
  button-brand:
    backgroundColor: "rgba(6,182,212,0.1)"
    textColor: "{colors.brand}"
    rounded: "{rounded.md}"
    padding: 8px
  badge-neutral:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.secondary}"
    rounded: "{rounded.sm}"
    padding: 4px
  card-elevated:
    backgroundColor: "{colors.surface-elevated}"
    textColor: "{colors.primary}"
    rounded: "{rounded.lg}"
    padding: 20px
  input-field:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.primary}"
    rounded: "{rounded.md}"
    padding: 8px
---

## Overview

CADS (CATerm Design System) v1.0 establishes the visual identity, tokens, and component contract for CATerm and CA Group developer utilities.

### Brand & Ethos
- **Sovereign Developer First:** Zero telemetry leaks, zero-knowledge storage, local-first architecture.
- **Cyberpunk Monoline Wireframe & Tech-Brutalist HUD:** Deep `#0A0A0C` surface base, hollow negative space, crisp 1px borders, subtle halos, and high-readability monochrome typography.
- **Anti-Bloat:** Strict component size scale, high performance Svelte 5 runes (`$state`, `$props`, `$derived`), zero heavy multi-color gradients.

## Colors

The core color palette is divided into monochrome foundational surfaces and dynamic Pro accent variables:

### Core Palette Tokens
- **Primary (`#EDEDED`):** High-contrast text, headings, and primary solid confirm button fills.
- **Secondary (`#A1A1AA`):** Captions, secondary labels, icon fills, and muted metadata.
- **Neutral (`#18181B`):** Elevated interactive elements, input background fills, and badge foundations.
- **Surface (`#0A0A0C`):** Base workspace canvas, dark window shell, and sidebar background.
- **Surface Elevated (`#121217`):** Card panels, popups, command palettes, and modal backgrounds.
- **Surface Subtle (`#18181F`):** Hover states, chip backgrounds, and table row zebra striping.
- **Border (`#272732`):** Monoline 1px HUD wireframe separator lines.

### Dynamic Brand Accent (`--ca-brand`)
The Pro tier activates customizable accent theming via CSS variables:
- **Cyan (`#06B6D4`):** Default CATerm terminal and CAMark telemetry.
- **Emerald (`#10B981`):** CACash fintech and transaction ledger tools.
- **Violet (`#8B5CF6`):** CAStudio content automation and media pipeline.
- **Rose (`#F43F5E`):** Critical alerts, audit exceptions, and destructive actions.
- **Blue (`#3B82F6`):** Enterprise cluster control plane.

## Typography

- **Headings & Body UI:** Inter / System UI stack (`font-sans`), optimized for crisp rendering on dark backgrounds.
- **Code & Diagnostics:** JetBrains Mono (`font-mono`), with tabular figures (`tabular-nums`) for IP addresses, latency, ports, memory metrics, and timestamps.
- **Hierarchy Scale:**
  - `h1`: 24px / 1.5rem, bold, -0.02em letter-spacing.
  - `h2`: 20px / 1.25rem, bold, -0.01em letter-spacing.
  - `h3`: 14px / 0.875rem, bold.
  - `body-md`: 12px / 0.75rem, regular, 1.5 line-height.
  - `mono-sm`: 11px / 0.6875rem, medium, tabular numerals.

## Layout

- **8pt HUD Grid System:** All paddings, margins, and gaps align to 4px/8px modular increments (`xs: 4px`, `sm: 8px`, `md: 16px`, `lg: 24px`, `xl: 32px`).
- **Workspace Density:** Compact information layout engineered for developer density without visual clutter.

## Elevation & Depth

- **Surface Base (`#0A0A0C`):** Lowest layer (terminal split panes and background).
- **Surface Elevated (`#121217`):** Raised container layer with 1px border (`#272732`).
- **Halo & Glow:** Restricted to subtle state highlights (e.g. active tab indicator, Pro halo `shadow-xs`). Heavy blurred drop shadows are strictly prohibited.

## Shapes

- **Corner Radius Scale:**
  - `sm (6px)`: Badges, tags, and micro tooltips.
  - `md (8px)`: Buttons, text inputs, dropdown menus, and tabs.
  - `lg (12px)`: Cards, interactive widgets, and dialog sections.
  - `xl (16px)`: Application window modal shells.

## Components

The Svelte 5 component suite resides in `frontend/src/lib/components/ui/`:

### 1. `Button.svelte`
- **Variants:** `primary` (solid white confirm), `secondary` (subtle dark), `outline` (monoline wireframe), `ghost` (flat hover), `danger` (alert outline), `brand` (dynamic `--ca-brand` outline).
- **Size Scale:** `sm` (28px height), `md` (36px height), `lg` (42px height), `icon` (square icon button).

### 2. `Badge.svelte`
- **Variants:** `neutral`, `success`, `warning`, `danger`, `brand`.
- **Sizes:** `xs` (10px monospace micro badge), `sm` (12px standard status chip).

### 3. `Input.svelte`
- **Features:** Dark background (`#18181B`), 1px border (`#27272A`), focus ring, optional leading icon snippet, trailing action snippet, full two-way `$bindable` value support.

### 4. `Card.svelte`
- **Features:** Elevated container (`#121217`), 1px border (`#272732`), header with title & description, customizable header actions slot, and clean card body content slot.

## Do's and Don'ts

### Do's
- Use solid white buttons exclusively for high-emphasis primary confirmations.
- Keep terminal and data displays strictly aligned using monospace and tabular numerals.
- Use monoline wireframe outlines (`outline` variant) for utility toolbars.
- Use token references and CSS custom properties (`--ca-brand`) for all theme adaptations.

### Don'ts
- Do not use heavy saturated gradient fills or blurred background shadows.
- Do not mix language strings hardcoded in UI; use the `$lib/i18n` translation contract.
- Do not override base button heights outside the `sm`/`md`/`lg` tokens.
