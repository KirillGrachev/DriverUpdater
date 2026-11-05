import { invoke } from '@tauri-apps/api/core';
import type { Backend } from './backend';
import type {
    CheckUpdatesResult,
    Driver,
    DriverBackup,
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

    getInstalledComponents(): Promise<string[]> {
        return invoke<string[]>('get_installed_components');
    }

    saveInstalledComponents(ids: string[]): Promise<string[]> {
        return invoke<string[]>('save_installed_components', { ids });
    }

    listPackageVersions(packageId: string): Promise<string[]> {
        return invoke<string[]>('list_package_versions', { packageId });
    }

    installPackage(packageId: string, version: string): Promise<void> {
        return invoke<void>('install_package', { packageId, version });
    }

    cancelPackageInstall(packageId: string): Promise<boolean> {
        return invoke<boolean>('cancel_package_install', { packageId });
    }

    listDriverBackups(): Promise<DriverBackup[]> {
        return invoke<DriverBackup[]>('list_driver_backups');
    }

    createDriverBackup(): Promise<DriverBackup> {
        return invoke<DriverBackup>('create_driver_backup');
    }

    restoreDriverBackup(id: string): Promise<void> {
        return invoke<void>('restore_driver_backup', { id });
    }

    deleteDriverBackup(id: string): Promise<void> {
        return invoke<void>('delete_driver_backup', { id });
    }

    getSystemInfo(): Promise<SystemInfo> {
        return invoke<SystemInfo>('get_system_info');
    }

    computeSummary(drivers: Driver[]): Promise<Summary> {
        return invoke<Summary>('compute_summary', { drivers });
    }
}
