import { invoke } from '@tauri-apps/api/core';
import type { Backend } from './backend';
import type {
  CheckUpdatesResult,
  Driver,
  InitialData,
  InstalledComponent,
  RestorePoint,
  Settings,
  Summary,
  SystemInfo,
} from './types';

/** Real IPC bridge to the Rust core (`src-tauri/src/commands.rs`). */
export class TauriBackend implements Backend {
  getInitialData(): Promise<InitialData> {
    return invoke<InitialData>('get_initial_data');
  }

  checkUpdates(): Promise<CheckUpdatesResult> {
    return invoke<CheckUpdatesResult>('check_updates');
  }

  installDriver(id: string): Promise<Driver> {
    return invoke<Driver>('install_driver', { id });
  }

  installUpdates(ids: string[]): Promise<Driver[]> {
    return invoke<Driver[]>('install_updates', { ids });
  }

  listRestorePoints(): Promise<RestorePoint[]> {
    return invoke<RestorePoint[]>('list_restore_points');
  }

  createRestorePoint(name: { ru: string; en: string }): Promise<RestorePoint> {
    return invoke<RestorePoint>('create_restore_point', { name });
  }

  deleteRestorePoint(id: string): Promise<void> {
    return invoke<void>('delete_restore_point', { id });
  }

  restoreFromPoint(id: string): Promise<void> {
    return invoke<void>('restore_from_point', { id });
  }

  saveSettings(settings: Settings): Promise<Settings> {
    return invoke<Settings>('save_settings', { settings });
  }

  openBackupFolder(): Promise<void> {
    return invoke<void>('open_backup_folder');
  }

  detectInstalledComponents(): Promise<InstalledComponent[]> {
    return invoke<InstalledComponent[]>('detect_installed_components');
  }

  getSystemInfo(): Promise<SystemInfo> {
    return invoke<SystemInfo>('get_system_info');
  }

  computeSummary(drivers: Driver[]): Promise<Summary> {
    return invoke<Summary>('compute_summary', { drivers });
  }
}
