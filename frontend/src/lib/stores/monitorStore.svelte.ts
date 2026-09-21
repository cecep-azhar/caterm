import { pollActiveMetrics, type HostMetrics } from '../api/monitor';

export const monitorState = $state({
    metrics: [] as HostMetrics[],
    lastUpdated: null as Date | null,
    isPolling: false
});

let timer: ReturnType<typeof setInterval> | null = null;

export async function fetchMetrics() {
    if (typeof document !== 'undefined' && document.hidden) return;
    try {
        monitorState.isPolling = true;
        const res = await pollActiveMetrics();
        monitorState.metrics = res;
        monitorState.lastUpdated = new Date();
    } catch (e) {
        console.error("Monitor poll failed:", e);
    } finally {
        monitorState.isPolling = false;
    }
}

export function startMonitoring(intervalMs = 5000) {
    if (timer) return;
    fetchMetrics();
    timer = setInterval(fetchMetrics, intervalMs);
}

export function stopMonitoring() {
    if (timer) {
        clearInterval(timer);
        timer = null;
    }
}
