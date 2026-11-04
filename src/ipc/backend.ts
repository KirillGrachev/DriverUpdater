import type {
  CheckUpdatesResult,
  Driver,
  InitialData,
  RestorePoint,
  Settings,
  Summary,
  SystemInfo,
} from './types';

/**
 * Transport-agnostic backend contract.
 *
 * Two implementations exist:
 *  - `tauri.ts`  — invokes the Rust core over IPC (real desktop app);
 *  - `mock.ts`   — in-browser mock backed by the same fixtures, used for
 *                  web previews and unit tests until the core is available.
 */
export interface Backend {
  getInitialData(): Promise<InitialData>;
  checkUpdates(): Promise<CheckUpdatesResult>;
  installDriver(id: string): Promise<Driver>;
  installUpdates(ids: string[]): Promise<Driver[]>;
  listRestorePoints(): Promise<RestorePoint[]>;
  createRestorePoint(name: { ru: string; en: string }): Promise<RestorePoint>;
  deleteRestorePoint(id: string): Promise<void>;
  restoreFromPoint(id: string): Promise<void>;
  saveSettings(settings: Settings): Promise<Settings>;
  getSystemInfo(): Promise<SystemInfo>;
  computeSummary(drivers: Driver[]): Promise<Summary>;
}

export function isTauriRuntime(): boolean {
  return (
    typeof window !== 'undefined' &&
    '__TAURI_INTERNALS__' in window &&
    window.__TAURI_INTERNALS__ !== undefined
  );
}

export async function createBackend(): Promise<Backend> {
  if (isTauriRuntime()) {
    const { TauriBackend } = await import('./tauri');
    return new TauriBackend();
  }
  const { MockBackend } = await import('./mock');
  return new MockBackend();
}
