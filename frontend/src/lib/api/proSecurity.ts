import { invoke } from '@tauri-apps/api/core';

export type DangerLevel = 'warning' | 'critical' | 'blocked';

export interface BlastShieldRisk {
  matchedRule: string;
  level: DangerLevel;
  reason: string;
  suggestedAlternative?: string;
}

export interface BlastShieldCheckResult {
  isDestructive: boolean;
  command: string;
  risk?: BlastShieldRisk;
}

export type SecretType =
  | 'awsAccessKey'
  | 'gitHubToken'
  | 'openAiKey'
  | 'privateKey'
  | 'genericBearerToken'
  | 'passwordInUri';

export interface DetectedSecret {
  secretType: SecretType;
  title: string;
  description: string;
  matchedSnippetMasked: string;
  suggestedFix: string;
}

export interface PasteCheckResult {
  containsSecret: boolean;
  secrets: DetectedSecret[];
  sanitizedText: string;
  recommendedPrefix: string;
}

export type AuditSeverity = 'info' | 'warning' | 'high' | 'critical';

export interface LaptopPostureFinding {
  id: string;
  title: string;
  description: string;
  severity: AuditSeverity;
  recommendation: string;
  details?: string;
}

export interface LaptopPostureReport {
  timestamp: string;
  overallScore: number;
  findings: LaptopPostureFinding[];
  scannedSshKeys: number;
  scannedEnvFiles: number;
  scannedListeningPorts: number;
}

export type FileIntegrityStatus = 'unchanged' | 'modified' | 'missing' | 'permissionChanged';

export interface BaselineFileRecord {
  path: string;
  sha256Hash: string;
  mode: number;
  size: number;
}

export interface IntegrityCheckItem {
  path: string;
  status: FileIntegrityStatus;
  baselineHash: string;
  currentHash?: string;
  baselineMode: number;
  currentMode?: number;
  alertMessage?: string;
}

export interface IntegrityTripwireReport {
  targetName: string;
  timestamp: string;
  totalFilesMonitored: number;
  tamperedCount: number;
  items: IntegrityCheckItem[];
}

// ======================== BATCH 2 TYPINGS ========================

export interface EphemeralKeyRecord {
  id: string;
  name: string;
  publicKey: string;
  privateKeyPem: string;
  createdAt: number;
  expiresAt: number;
  ttlSeconds: number;
  commentSignature: string;
  targetHost?: string;
  autoRevoke: boolean;
  isRevoked: boolean;
}

export interface CreateEphemeralKeyRequest {
  name: string;
  ttlSeconds: number;
  targetHost?: string;
  autoRevoke?: boolean;
}

export interface EphemeralKeyStatus {
  id: string;
  name: string;
  publicKey: string;
  remainingSeconds: number;
  isExpired: boolean;
  isRevoked: boolean;
  commentSignature: string;
  targetHost?: string;
  createdAtIso: string;
  expiresAtIso: string;
}

export interface BastionHop {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  authType: string;
  keyId?: string;
}

export interface JumpChainConfig {
  id: string;
  name: string;
  targetHost: string;
  targetPort: number;
  targetUsername: string;
  hops: BastionHop[];
  createdAt: string;
}

export interface HopProbeStatus {
  hopIndex: number;
  name: string;
  host: string;
  port: number;
  reachable: boolean;
  latencyMs?: number;
  error?: string;
}

export interface JumpChainProbeResult {
  chainId: string;
  isFullyTraversable: boolean;
  hopStatuses: HopProbeStatus[];
  sshProxyCommand: string;
  sshJumpDirective: string;
}

export type DockerProbeSeverity = 'info' | 'warning' | 'high' | 'critical';

export interface DockerSecurityFinding {
  id: string;
  title: string;
  description: string;
  severity: DockerProbeSeverity;
  affectedResource: string;
  recommendation: string;
}

export interface ContainerAuditSummary {
  id: string;
  name: string;
  image: string;
  isPrivileged: boolean;
  hostPid: boolean;
  hostNetwork: boolean;
  sensitiveMounts: string[];
  exposedPorts: string[];
}

export interface DockerProbeReport {
  timestamp: string;
  dockerSocketFound: boolean;
  dockerSocketPath: string;
  socketPermissions?: string;
  tcpExposedUnauth: boolean;
  containersScanned: number;
  findings: DockerSecurityFinding[];
  containers: ContainerAuditSummary[];
  securityScore: number;
}

export interface CastChunk {
  timeOffset: number;
  eventType: string;
  data: string;
  prevHash: string;
  blockHash: string;
}

export interface AuditBlockReceipt {
  sessionId: string;
  totalChunks: number;
  totalBytes: number;
  startTimestamp: number;
  endTimestamp: number;
  genesisHash: string;
  terminalHash: string;
  isValid: boolean;
}

// ======================== API CLIENT INVOCATIONS ========================

export async function checkCommandBlastShield(command: string): Promise<BlastShieldCheckResult> {
  return await invoke<BlastShieldCheckResult>('pro_blast_shield_check', { command });
}

export async function checkPasteSentinel(text: string): Promise<PasteCheckResult> {
  return await invoke<PasteCheckResult>('pro_paste_sentinel_check', { text });
}

export async function scanLaptopPosture(): Promise<LaptopPostureReport> {
  return await invoke<LaptopPostureReport>('pro_laptop_posture_scan');
}

export async function createIntegrityBaseline(
  targetName: string,
  paths?: string[]
): Promise<BaselineFileRecord[]> {
  return await invoke<BaselineFileRecord[]>('pro_integrity_create_baseline', {
    targetName,
    paths: paths ?? null
  });
}

export async function verifyIntegrityTripwire(
  targetName: string,
  baseline: BaselineFileRecord[]
): Promise<IntegrityTripwireReport> {
  return await invoke<IntegrityTripwireReport>('pro_integrity_verify', {
    targetName,
    baseline
  });
}

export async function createEphemeralKey(
  request: CreateEphemeralKeyRequest
): Promise<EphemeralKeyRecord> {
  return await invoke<EphemeralKeyRecord>('pro_ephemeral_key_create', { request });
}

export async function listEphemeralKeys(): Promise<EphemeralKeyStatus[]> {
  return await invoke<EphemeralKeyStatus[]>('pro_ephemeral_keys_list');
}

export async function revokeEphemeralKey(id: string): Promise<boolean> {
  return await invoke<boolean>('pro_ephemeral_key_revoke', { id });
}

export async function probeJumpChain(config: JumpChainConfig): Promise<JumpChainProbeResult> {
  return await invoke<JumpChainProbeResult>('pro_jump_proxy_probe', { config });
}

export async function runDockerProbe(
  remoteSocketInfo?: [string, number],
  containers?: any[]
): Promise<DockerProbeReport> {
  return await invoke<DockerProbeReport>('pro_docker_probe_scan', {
    remoteSocketInfo: remoteSocketInfo ?? null,
    containers: containers ?? null
  });
}

export async function verifySessionAuditChain(
  genesisHash: string,
  chunks: CastChunk[]
): Promise<boolean> {
  return await invoke<boolean>('pro_session_audit_verify', { genesisHash, chunks });
}
