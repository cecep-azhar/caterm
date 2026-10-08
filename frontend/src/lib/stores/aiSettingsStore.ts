import { writable } from 'svelte/store';
import {
  type AiProvider,
  type AiProviderConfig,
  type TaskRouteRule,
  type SystemPersona,
  type HabitFact,
  aiGetProviders,
  aiSaveProvider,
  aiDeleteProvider,
  aiGetRoutingMatrix,
  aiSaveRoutingRule,
  aiGetPersonas,
  aiSavePersona,
  aiDeletePersona,
  aiGetHabits,
  aiSearchHabits,
  aiToggleHabitPin,
  aiDeleteHabit
} from '$lib/api/aiRouting';

export interface AiSettingsState {
  providers: AiProvider[];
  routingMatrix: TaskRouteRule[];
  personas: SystemPersona[];
  habits: HabitFact[];
  loading: boolean;
  error: string | null;
}

const initialState: AiSettingsState = {
  providers: [],
  routingMatrix: [],
  personas: [],
  habits: [],
  loading: false,
  error: null
};

export const aiSettingsStore = writable<AiSettingsState>(initialState);

export async function loadAiData(): Promise<void> {
  aiSettingsStore.update((s) => ({ ...s, loading: true, error: null }));
  try {
    const [providers, routingMatrix, personas, habits] = await Promise.all([
      aiGetProviders(),
      aiGetRoutingMatrix(),
      aiGetPersonas(),
      aiGetHabits()
    ]);
    aiSettingsStore.update((s) => ({
      ...s,
      providers,
      routingMatrix,
      personas,
      habits,
      loading: false
    }));
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    aiSettingsStore.update((s) => ({ ...s, loading: false, error: message }));
  }
}

export async function reloadProviders(): Promise<void> {
  try {
    const providers = await aiGetProviders();
    aiSettingsStore.update((s) => ({ ...s, providers }));
  } catch (err) {
    console.error('Failed to reload providers', err);
  }
}

export async function persistProvider(provider: AiProviderConfig): Promise<void> {
  await aiSaveProvider(provider);
  await reloadProviders();
}

export async function removeProvider(id: string): Promise<void> {
  await aiDeleteProvider(id);
  await reloadProviders();
}

export async function reloadRoutingMatrix(): Promise<void> {
  try {
    const routingMatrix = await aiGetRoutingMatrix();
    aiSettingsStore.update((s) => ({ ...s, routingMatrix }));
  } catch (err) {
    console.error('Failed to reload routing matrix', err);
  }
}

export async function persistRoutingRule(rule: TaskRouteRule): Promise<void> {
  await aiSaveRoutingRule(rule);
  await reloadRoutingMatrix();
}

export async function reloadPersonas(): Promise<void> {
  try {
    const personas = await aiGetPersonas();
    aiSettingsStore.update((s) => ({ ...s, personas }));
  } catch (err) {
    console.error('Failed to reload personas', err);
  }
}

export async function persistPersona(persona: SystemPersona): Promise<void> {
  await aiSavePersona(persona);
  await reloadPersonas();
}

export async function removePersona(id: string): Promise<void> {
  await aiDeletePersona(id);
  await reloadPersonas();
}

export async function reloadHabits(): Promise<void> {
  try {
    const habits = await aiGetHabits();
    aiSettingsStore.update((s) => ({ ...s, habits }));
  } catch (err) {
    console.error('Failed to reload habits', err);
  }
}

export async function searchHabits(query: string): Promise<HabitFact[]> {
  if (!query.trim()) {
    const habits = await aiGetHabits();
    aiSettingsStore.update((s) => ({ ...s, habits }));
    return habits;
  }
  const habits = await aiSearchHabits(query);
  aiSettingsStore.update((s) => ({ ...s, habits }));
  return habits;
}

export async function toggleHabitPin(id: string): Promise<boolean> {
  const isPinned = await aiToggleHabitPin(id);
  aiSettingsStore.update((s) => ({
    ...s,
    habits: s.habits.map((h) => (h.id === id ? { ...h, is_pinned: isPinned } : h))
  }));
  return isPinned;
}

export async function removeHabit(id: string): Promise<void> {
  await aiDeleteHabit(id);
  aiSettingsStore.update((s) => ({
    ...s,
    habits: s.habits.filter((h) => h.id !== id)
  }));
}
