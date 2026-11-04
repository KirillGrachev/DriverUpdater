import driversFixture from '../../shared/fixtures/drivers.json';
import metaFixture from '../../shared/fixtures/meta.json';
import restoreFixture from '../../shared/fixtures/restore_points.json';
import settingsFixture from '../../shared/fixtures/settings.json';
import type { Backend } from './backend';
import type {
  CheckUpdatesResult,
  Driver,
  InitialData,
  LocalizedText,
  RestorePoint,
  Settings,
  Summary,
  SystemInfo,
} from './types';

const STORAGE_KEY = 'driver-updater.mock.v1';

interface MockState {
  drivers: Driver[];
  restorePoints: RestorePoint[];
  settings: Settings;
  lastCheck: string | null;
}

const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const latency = (base = 140, spread = 220) => sleep(base + Math.random() * spread);
const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value)) as T;

function defaultState(): MockState {
  return {
    drivers: clone(driversFixture) as Driver[],
    restorePoints: clone(restoreFixture) as RestorePoint[],
    settings: clone(settingsFixture) as Settings,
    lastCheck: new Date().toISOString(),
  };
}

function loadState(): MockState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return defaultState();
    const parsed = JSON.parse(raw) as MockState;
    if (!parsed.drivers?.length || !parsed.settings) return defaultState();
    return parsed;
  } catch {
    return defaultState();
  }
}

function detectSystemInfo(): SystemInfo {
  const ua = navigator.userAgent;
  let os = 'Unknown OS';
  if (/Windows NT 10\.0/.test(ua)) os = 'Windows 11 (10.0.26100)';
  else if (/Windows NT/.test(ua)) os = 'Windows (legacy)';
  else if (/Mac OS X/.test(ua)) os = 'macOS';
  else if (/Android/.test(ua)) os = 'Android';
  else if (/Linux/.test(ua)) os = 'Linux';
  const arch = /x86_64|x64|Win64|WOW64/.test(ua) ? 'x64' : /arm64|aarch64/i.test(ua) ? 'arm64' : 'x86';
  return { os, arch, platform: navigator.platform || 'Web' };
}

/**
 * In-browser backend used for web previews and tests.
 * Persists mutable state in localStorage so the demo behaves consistently.
 */
export class MockBackend implements Backend {
  private state: MockState = loadState();

  private persist(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.state));
    } catch {
      /* storage may be unavailable (private mode) — ignore */
    }
  }

  private summary(): Summary {
    const updatesAvailable = this.state.drivers.filter((d) => d.status === 'update').length;
    return {
      devicesTotal: metaFixture.devicesTotal,
      upToDate: metaFixture.devicesTotal - updatesAvailable,
      updatesAvailable,
      lastCheck: this.state.lastCheck,
    };
  }

  async getInitialData(): Promise<InitialData> {
    await latency();
    return {
      drivers: clone(this.state.drivers),
      restorePoints: clone(this.state.restorePoints),
      settings: clone(this.state.settings),
      summary: this.summary(),
      systemInfo: detectSystemInfo(),
    };
  }

  async checkUpdates(): Promise<CheckUpdatesResult> {
    await sleep(900 + Math.random() * 500);
    this.state.lastCheck = new Date().toISOString();
    this.persist();
    return { drivers: clone(this.state.drivers), summary: this.summary() };
  }

  async installDriver(id: string): Promise<Driver> {
    await sleep(1100 + Math.random() * 500);
    const driver = this.state.drivers.find((d) => d.id === id);
    if (!driver) throw new Error(`Unknown driver: ${id}`);
    if (driver.available) driver.current = driver.available;
    driver.available = null;
    driver.status = 'current';
    this.persist();
    return clone(driver);
  }

  async installUpdates(ids: string[]): Promise<Driver[]> {
    await sleep(1500 + Math.random() * 700);
    const updated: Driver[] = [];
    for (const id of ids) {
      const driver = this.state.drivers.find((d) => d.id === id);
      if (!driver) continue;
      if (driver.available) driver.current = driver.available;
      driver.available = null;
      driver.status = 'current';
      updated.push(clone(driver));
    }
    this.state.lastCheck = new Date().toISOString();
    this.persist();
    return updated;
  }

  async listRestorePoints(): Promise<RestorePoint[]> {
    await latency();
    return clone(this.state.restorePoints);
  }

  async createRestorePoint(name: LocalizedText): Promise<RestorePoint> {
    await sleep(700 + Math.random() * 400);
    const point: RestorePoint = {
      id: `rp-${Date.now()}`,
      name,
      kind: 'manual',
      createdAt: new Date().toISOString(),
      sizeMb: Math.round(380 + Math.random() * 120),
    };
    this.state.restorePoints.unshift(point);
    this.persist();
    return clone(point);
  }

  async deleteRestorePoint(id: string): Promise<void> {
    await latency();
    this.state.restorePoints = this.state.restorePoints.filter((p) => p.id !== id);
    this.persist();
  }

  async restoreFromPoint(id: string): Promise<void> {
    await sleep(1000 + Math.random() * 500);
    if (!this.state.restorePoints.some((p) => p.id === id)) {
      throw new Error(`Unknown restore point: ${id}`);
    }
  }

  async saveSettings(settings: Settings): Promise<Settings> {
    await latency();
    this.state.settings = clone(settings);
    this.persist();
    return clone(settings);
  }

  async getSystemInfo(): Promise<SystemInfo> {
    await latency(60, 60);
    return detectSystemInfo();
  }

  async computeSummary(): Promise<Summary> {
    return this.summary();
  }
}
