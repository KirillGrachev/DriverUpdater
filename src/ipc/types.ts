/**
 * Shared IPC contracts between the Tauri core and the React webview.
 *
 * <p>The desktop build uses the real Windows detection backend. The browser
 * preview keeps the same contracts backed by {@code shared/fixtures/*.json}
 * so the UI can be developed without the native shell.</p>
 */

export type LocalizedText = { ru: string; en: string };

export type DriverCategory = 'video' | 'audio' | 'chipset' | 'network' | 'input';

export type DriverStatus = 'update' | 'optional' | 'current';

export type DriverIconKind = 'gpu' | 'audio' | 'chip' | 'network' | 'display' | 'input';

export interface Driver {
    id: string;
    name: string;
    vendor: string;
    category: DriverCategory;
    icon: { kind: DriverIconKind; color: string };
    current: string;
    available: string | null;
    status: DriverStatus;
    sizeMb: number;
    recommended: boolean;
    device: string;
    released: string;
    description: LocalizedText;
    /** Device instance id from Windows PnP. */
    deviceId?: string;
    /** Hardware ids reported by the driver. */
    hardwareIds?: string[];
    /** Published INF name when available. */
    infName?: string;
    /** Windows Update identity used to install this update. */
    updateId?: string;
    rebootRequired?: boolean;
}

export type RestorePointKind = 'auto' | 'schedule' | 'manual';

export interface RestorePoint {
    id: string;
    name: LocalizedText;
    kind: RestorePointKind;
    createdAt: string;
    sizeMb: number;
}

export type ThemeMode = 'light' | 'dark' | 'system';
export type CheckInterval = 'hourly' | 'daily' | 'weekly';
export type RestorePointPeriod = 'daily' | 'weekly' | 'monthly';

export interface Settings {
    language: string;
    theme: ThemeMode;
    notifications: boolean;
    autoCheck: boolean;
    checkInterval: CheckInterval;
    restorePointBeforeInstall: boolean;
    keepBackups: boolean;
    backupPath: string;
    telemetry: boolean;
    autoRestorePoints: boolean;
    restorePointPeriod: RestorePointPeriod;
}

export interface Summary {
    devicesTotal: number;
    upToDate: number;
    updatesAvailable: number;
    /** ISO string of the last successful scan, null if never scanned. */
    lastCheck: string | null;
}

export interface SystemInfo {
    os: string;
    arch: string;
    platform: string;
    hostname: string;
}

export interface DriverBackup {
    id: string;
    createdAt: string;
    path: string;
    sizeMb: number;
    packageCount: number;
}

export interface InstalledComponent {
    id: string;
    version: string;
    path: string;
}

/** Payload returned by the core on application start. */
export interface InitialData {
    drivers: Driver[];
    restorePoints: RestorePoint[];
    settings: Settings;
    summary: Summary;
    systemInfo: SystemInfo;
    scanError?: string | null;
}

export interface CheckUpdatesResult {
    drivers: Driver[];
    summary: Summary;
}
