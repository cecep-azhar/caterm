import { invoke } from '@tauri-apps/api/core';

export interface EgressPortResult {
  port: number;
  target: string;
  serviceLabel: string;
  status: 'BLOCKED' | 'OPEN_LEAK' | 'TIMEOUT' | 'SKIPPED' | string;
  riskLevel: 'CRITICAL' | 'HIGH' | 'MEDIUM' | 'LOW' | string;
  notes: string;
}

export interface InboundListeningPort {
  proto: string;
  localAddress: string;
  port: number;
  process: string;
  isWildcard: boolean;
  riskLevel: 'CRITICAL' | 'HIGH' | 'LOW' | string;
  recommendation: string;
}

export interface LatencyProbeResult {
  target: string;
  packetLossPct: number;
  avgLatencyMs: number;
  minLatencyMs: number;
  maxLatencyMs: number;
  hops: string[];
  status: 'EXCELLENT' | 'FAIR' | 'DEGRADED' | string;
}

export interface HardeningCheckItem {
  name: string;
  category: 'SSH' | 'FIREWALL' | 'KERNEL' | string;
  currentValue: string;
  recommendedValue: string;
  status: 'PASS' | 'WARN' | 'FAIL' | string;
  details: string;
}

export interface AuditReport {
  hostId: string;
  timestamp: number;
  egressResults: EgressPortResult[];
  inboundPorts: InboundListeningPort[];
  latencyProbe: LatencyProbeResult;
  hardeningChecklist: HardeningCheckItem[];
  summaryScore: number;
}

export async function runNetworkSecurityAudit(hostId: string): Promise<AuditReport> {
  return invoke<AuditReport>('run_network_security_audit', { hostId, host_id: hostId });
}
