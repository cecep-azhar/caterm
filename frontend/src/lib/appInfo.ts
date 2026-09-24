// Single source for the version string the UI prints. It used to be hardcoded in four places
// and had drifted (sidebar said v2.1.7 while the build was 2.1.8). package.json is bumped in
// lockstep with the Cargo workspace / tauri.conf.json on release.
import { version } from '../../package.json';

export const APP_VERSION: string = version;
export const REPO_URL = 'https://github.com/cecep-azhar/caterm';

export function releaseNotesUrl(ver: string): string {
  return `${REPO_URL}/releases/tag/v${ver.replace(/^v/, '')}`;
}
