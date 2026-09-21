import { invoke } from '@tauri-apps/api/core';

export interface AiPlanStep {
  id?: string;
  step_number: number;
  title: string;
  command: string;
  description: string;
  is_sudo?: boolean;
  is_danger?: boolean;
}

export interface AiExecutionPlan {
  id?: string;
  summary: string;
  requirements: string[];
  steps: AiPlanStep[];
  estimated_time?: string;
  /** 'llm' when the model produced it, 'builtin' when the offline template planner did. */
  source?: 'llm' | 'builtin' | string;
}

export interface AiExecutionResult {
  step_id?: string;
  step_number?: number;
  success: boolean;
  stdout: string;
  stderr: string;
  exit_code?: number;
  duration_ms?: number;
}

export interface AiSettings {
  provider: string;
  base_url: string;
  api_key: string;
  model: string;
}

export async function getAiSettings(): Promise<AiSettings> {
  return invoke<AiSettings>('get_ai_settings');
}

export async function saveAiSettings(settings: AiSettings): Promise<void> {
  return invoke<void>('save_ai_settings', { settings, input: settings });
}

export async function aiGeneratePlan(goal: string, hostId?: string): Promise<AiExecutionPlan> {
  return invoke<AiExecutionPlan>('ai_generate_plan', {
    goal,
    hostId,
    prompt: goal,
    host_id: hostId
  });
}

export async function aiExecuteStep(
  hostId: string,
  step: AiPlanStep | string
): Promise<AiExecutionResult> {
  const command = typeof step === 'string' ? step : step.command;
  const stepObj = typeof step === 'string'
    ? { command, step_number: 1, title: 'Execute', description: '', is_sudo: command.includes('sudo') }
    : step;

  return invoke<AiExecutionResult>('ai_execute_step', {
    hostId,
    host_id: hostId,
    step: stepObj,
    command
  });
}

export interface AiChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface AiChatReply {
  reply: string;
  /** True only once the assistant has stopped asking questions and proposes commands. */
  ready: boolean;
  steps: AiPlanStep[];
}

/**
 * One turn of conversation. Send the whole history each time — the backend is stateless and
 * the model needs the earlier answers to know what it already asked.
 */
export async function aiChat(
  messages: AiChatMessage[],
  hostLabel?: string
): Promise<AiChatReply> {
  return invoke<AiChatReply>('ai_chat', { messages, hostLabel });
}
