import { invoke } from '@tauri-apps/api/core';

/** `rating` 1-5, `content` feedback text. Validated in caterm-core before HTTP. */
export function submitFeedback(rating: number, content: string): Promise<void> {
  return invoke('submit_feedback', { rating, content });
}
