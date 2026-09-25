// Manually curated for now: no backend proxy exists yet to safely call the GitHub Sponsors
// API from this desktop client (it needs an authenticated token, which can't live in client
// code). Update this list by hand as sponsors come in. Once a proxy endpoint exists (fathforce
// monorepo), this can be swapped for a fetched list without touching SponsorWall.svelte.
export type SponsorTierId = 'platinum' | 'gold' | 'silver' | 'contributor';

export interface SponsorTier {
  id: SponsorTierId;
  /** Minimum monthly USD to qualify; 'contributor' has no floor (any amount, incl. one-time). */
  minUsd: number | null;
  from: string;
  to: string;
}

export const SPONSOR_TIERS: SponsorTier[] = [
  { id: 'platinum', minUsd: 100, from: '#e5e7eb', to: '#9ca3af' },
  { id: 'gold', minUsd: 50, from: '#fde68a', to: '#d97706' },
  { id: 'silver', minUsd: 10, from: '#cbd5e1', to: '#64748b' },
  { id: 'contributor', minUsd: null, from: '#7dd3fc', to: '#0ea5e9' }
];

export interface Sponsor {
  name: string;
  tier: SponsorTierId;
  url?: string;
  avatarUrl?: string;
}

export const SPONSORS: Sponsor[] = [];
