#!/usr/bin/env bash
# T2-BOOT-06 budget gate. Reads bench-report.json (from scripts/bench.sh) and fails
# (exit != 0) if any Fase-0-measurable REQ-02 metric exceeds its threshold. Thresholds
# are overridable via CATERM_BUDGET_* env vars so the gate's bite can be proven on demand.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT="${1:-$PROJECT_ROOT/bench-report.json}"

if [ ! -f "$REPORT" ]; then
    echo "bench-report.json tidak ada di $REPORT - jalankan scripts/bench.sh dulu" >&2
    exit 2
fi

extract_num() {
    grep "\"$1\"" "$REPORT" | head -1 | sed -E 's/[^:]*:[[:space:]]*([0-9]+).*/\1/'
}

BINARY_SIZE_BYTES=$(extract_num binary_size_bytes)
COLD_START_MS=$(extract_num cold_start_ms)
RSS_BYTES=$(extract_num rss_bytes)

# RSS default direvisi 2026-09-18 (T2-BOOT-06, Q-01): baseline app kosong terukur nyata
# ~330 MB di Windows/WebView2 - jauh di atas angka lama 35 MB. Lihat prd-v2.md REQ-02 dan
# qa-report-v2.md untuk rasional lengkap. Revisi ini butuh konfirmasi pemilik (non-blocking).
BUDGET_BINARY_MB="${CATERM_BUDGET_BINARY_MB:-8}"
BUDGET_COLD_START_MS="${CATERM_BUDGET_COLD_START_MS:-800}"
BUDGET_RSS_MB="${CATERM_BUDGET_RSS_MB:-380}"

# Round UP to the nearest whole MB - never let integer truncation hide an overage.
BINARY_SIZE_MB=$(((BINARY_SIZE_BYTES + 1048575) / 1048576))
RSS_MB=$(((RSS_BYTES + 1048575) / 1048576))

FAIL=0

check() {
    local name="$1" actual="$2" budget="$3" unit="$4"
    if [ "$actual" -gt "$budget" ]; then
        echo "GAGAL: $name = $actual $unit > ambang $budget $unit"
        FAIL=1
    else
        echo "OK: $name = $actual $unit <= ambang $budget $unit"
    fi
}

check "binary_size_mb" "$BINARY_SIZE_MB" "$BUDGET_BINARY_MB" "MB"
check "cold_start_ms" "$COLD_START_MS" "$BUDGET_COLD_START_MS" "ms"
check "rss_mb (RSS aktif empty-app proxy)" "$RSS_MB" "$BUDGET_RSS_MB" "MB"

echo "db_query_ms: belum tersedia - tabel hosts belum ada (Fase 1, T2-CORE-01), tidak dievaluasi"

exit $FAIL
