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

// DevOps Diagnostics & Lab Suite Data Interfaces

export interface ServerBenchmarkResult {
  hostId: string;
  timestamp: number;
  cpuCores: number;
  cpuSingleScore: number;
  cpuMultiScore: number;
  ramBandwidthMbps: number;
  diskWriteIops4k: number;
  diskWriteMbps64k: number;
  diskWriteMbps1m: number;
  details: string;
}

export interface SpeedTestResult {
  hostId: string;
  timestamp: number;
  serverTarget: string;
  pingMs: number;
  jitterMs: number;
  downloadMbps: number;
  uploadMbps: number;
  status: string;
}

export interface QosAuditResult {
  hostId: string;
  timestamp: number;
  idleLatencyMs: number;
  downloadLoadLatencyMs: number;
  uploadLoadLatencyMs: number;
  bufferbloatGrade: 'A+' | 'A' | 'B' | 'C' | 'D' | 'F' | string;
  deltaLoadMs: number;
  recommendation: string;
}

export interface MeshNodePing {
  targetHost: string;
  targetLabel: string;
  reachable: boolean;
  rttMs: number;
  packetLossPct: number;
}

export interface MeshLatencyMatrix {
  sourceHostId: string;
  timestamp: number;
  nodes: MeshNodePing[];
}

export interface PmtudResult {
  hostId: string;
  targetIp: string;
  optimalMtu: number;
  hopsTested: number[];
  status: string;
  notes: string;
}

export interface SubnetDiscoveredHost {
  ip: string;
  isAlive: boolean;
  openPorts: number[];
  rttMs: number;
}

export interface SubnetSweepResult {
  hostId: string;
  cidr: string;
  totalScanned: number;
  activeHosts: number;
  hosts: SubnetDiscoveredHost[];
}

export interface SuspiciousOutboundSocket {
  pid: number;
  processName: string;
  exePath: string;
  cmdline: string;
  remoteAddress: string;
  severity: 'CRITICAL' | 'HIGH' | 'WARN' | string;
  reason: string;
}

export interface ThreatWatchdogResult {
  hostId: string;
  timestamp: number;
  suspiciousSockets: SuspiciousOutboundSocket[];
  totalAnalyzed: number;
  threatLevel: 'CLEAN' | 'WARNING' | 'COMPROMISED' | string;
  summary: string;
}

export interface TlsAuditResult {
  targetHost: string;
  targetPort: number;
  subject: string;
  issuer: string;
  validFrom: string;
  validTo: string;
  daysRemaining: number;
  cipherSuite: string;
  isExpired: boolean;
  status: 'HEALTHY' | 'EXPIRING_SOON' | 'EXPIRED' | 'ERROR' | string;
}

// API Invocation Wrappers

export async function runNetworkSecurityAudit(hostId: string): Promise<AuditReport> {
  return invoke<AuditReport>('run_network_security_audit', { hostId, host_id: hostId });
}

export async function runServerBenchmark(hostId: string): Promise<ServerBenchmarkResult> {
  return invoke<ServerBenchmarkResult>('run_server_benchmark', { hostId, host_id: hostId });
}

export async function runNetworkSpeedTest(hostId: string): Promise<SpeedTestResult> {
  return invoke<SpeedTestResult>('run_network_speed_test', { hostId, host_id: hostId });
}

export async function runQosAudit(hostId: string): Promise<QosAuditResult> {
  return invoke<QosAuditResult>('run_qos_audit', { hostId, host_id: hostId });
}

export async function runMeshLatencyMatrix(
  sourceHostId: string,
  targetHosts: [string, string][]
): Promise<MeshLatencyMatrix> {
  return invoke<MeshLatencyMatrix>('run_mesh_latency_matrix', {
    sourceHostId,
    source_host_id: sourceHostId,
    targetHosts,
    target_hosts: targetHosts
  });
}

export async function runPmtudProbe(hostId: string, targetIp: string): Promise<PmtudResult> {
  return invoke<PmtudResult>('run_pmtud_probe', {
    hostId,
    host_id: hostId,
    targetIp,
    target_ip: targetIp
  });
}

export async function runSubnetSweep(hostId: string, cidr: string): Promise<SubnetSweepResult> {
  return invoke<SubnetSweepResult>('run_subnet_sweep', {
    hostId,
    host_id: hostId,
    cidr
  });
}

export async function runThreatWatchdog(hostId: string): Promise<ThreatWatchdogResult> {
  return invoke<ThreatWatchdogResult>('run_threat_watchdog', { hostId, host_id: hostId });
}

export async function runTlsAudit(targetHost: string, port?: number): Promise<TlsAuditResult> {
  return invoke<TlsAuditResult>('run_tls_audit', {
    targetHost,
    target_host: targetHost,
    port
  });
}
