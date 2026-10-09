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
