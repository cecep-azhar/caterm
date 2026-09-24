// Single source of truth for version string prints. Was hardcoded in four places
// and drifted (sidebar said v2.1.7 while build was 2.1.8). package.json bumped in
// lockstep with Cargo workspace / tauri.conf.json on release.
import { version } from '../../package.json';

export const APP_VERSION: string = version;
export const REPO_URL = 'https://github.com/cecep-azhar/caterm';
export const WEBSITE_URL = 'https://caterm.fathforce.com';
export const AUTHOR_URL = 'https://cecepazhar.com';
export const PRICING_URL = `${WEBSITE_URL}/#pricing`;

export function releaseNotesUrl(ver: string): string {
  return `${REPO_URL}/releases/tag/v${ver.replace(/^v/, '')}`;
}
