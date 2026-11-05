import type {
    CheckUpdatesResult,
    Driver,
    DriverBackup,
    InitialData,
    InstalledComponent,
    InstallReport,
    RestorePoint,
    Settings,
    Summary,
    SystemInfo,
} from './types';

/**
 * Transport-agnostic backend contract.
 *
 * <p>Two implementations exist:</p>
 * <ul>
 *     <li>{@code tauri.ts} invokes the Rust core over IPC (desktop app);</li>
 *     <li>{@code mock.ts} is an in-browser backend backed by the same fixtures
 *     for previews and unit tests.</li>
 * </ul>
 */
export interface Backend {
    getInitialData(): Promise<InitialData>;
    checkUpdates(): Promise<CheckUpdatesResult>;
    installDriver(id: string, restorePointName: string): Promise<Driver>;
    installUpdates(ids: string[], restorePointName: string): Promise<InstallReport>;
    listRestorePoints(): Promise<RestorePoint[]>;
    createRestorePoint(name: string): Promise<RestorePoint>;
    deleteRestorePoint(id: string): Promise<void>;
    restoreFromPoint(id: string): Promise<void>;
    saveSettings(settings: Settings): Promise<Settings>;
    openBackupFolder(): Promise<void>;
    detectInstalledComponents(): Promise<InstalledComponent[]>;
    getInstalledComponents(): Promise<string[]>;
    saveInstalledComponents(ids: string[]): Promise<string[]>;
    listPackageVersions(packageId: string): Promise<string[]>;
    installPackage(packageId: string, version: string): Promise<void>;
    cancelPackageInstall(packageId: string): Promise<boolean>;
    listDriverBackups(): Promise<DriverBackup[]>;
    createDriverBackup(): Promise<DriverBackup>;
    restoreDriverBackup(id: string): Promise<void>;
    deleteDriverBackup(id: string): Promise<void>;
    getSystemInfo(): Promise<SystemInfo>;
    computeSummary(drivers: Driver[]): Promise<Summary>;
}

/**
 * Detects whether the page runs inside the Tauri webview rather than a
 * regular browser preview.
 *
 * @returns {@code true} when the Tauri IPC bridge is available
 */
export function isTauriRuntime(): boolean {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window && window.__TAURI_INTERNALS__ !== undefined;
}

/**
 * Creates the backend for the current runtime.
 *
 * @returns Tauri IPC backend in the desktop app, fixture-backed mock otherwise
 */
export async function createBackend(): Promise<Backend> {
    if (isTauriRuntime()) {
        const { TauriBackend } = await import('./tauri');
        return new TauriBackend();
    }
    if (import.meta.env.PROD) {
        throw new Error('Tauri IPC bridge is not available in a production build');
    }
    const { MockBackend } = await import('./mock');
    return new MockBackend();
}
