import { describe, expect, it } from 'vitest';
import type { Driver, Settings } from '../ipc/types';
import { autoCheckInterval, autoInstallIds, shouldAutoCheck } from './autoCheck';

const NOW = Date.parse('2026-11-07T00:00:00Z');

function settings(overrides: Partial<Settings> = {}): Settings {
    return {
        language: 'en',
        theme: 'system',
        autoCheck: true,
        autoInstall: false,
        checkInterval: 'daily',
        restorePointBeforeInstall: true,
        keepBackups: true,
        backupPath: '',
        autoRestorePoints: false,
        restorePointPeriod: 'weekly',
        ...overrides,
    };
}

describe('automatic update checks', () => {
    it('never runs when the setting is disabled', () => {
        expect(shouldAutoCheck(settings({ autoCheck: false }), '2020-01-01T00:00:00Z', NOW)).toBe(false);
    });

    it('runs once when no previous check exists', () => {
        expect(shouldAutoCheck(settings(), null, NOW)).toBe(true);
    });

    it('treats a malformed timestamp as due', () => {
        expect(shouldAutoCheck(settings(), 'not-a-date', NOW)).toBe(true);
    });

    it('respects daily, hourly and weekly intervals', () => {
        expect(autoCheckInterval('hourly')).toBe(60 * 60 * 1000);
        expect(autoCheckInterval('daily')).toBe(24 * 60 * 60 * 1000);
        expect(autoCheckInterval('weekly')).toBe(7 * 24 * 60 * 60 * 1000);
        expect(shouldAutoCheck(settings(), '2026-11-05T00:00:00Z', NOW)).toBe(true);
        expect(shouldAutoCheck(settings({ checkInterval: 'hourly' }), '2026-11-06T23:00:00Z', NOW)).toBe(true);
        expect(shouldAutoCheck(settings({ checkInterval: 'weekly' }), '2026-11-01T00:00:00Z', NOW)).toBe(false);
    });

    it('does not run before the interval elapses', () => {
        expect(shouldAutoCheck(settings(), '2026-11-06T12:00:00Z', NOW)).toBe(false);
    });

    it('runs at the exact boundary', () => {
        expect(shouldAutoCheck(settings(), '2026-11-06T00:00:00Z', NOW)).toBe(true);
    });
});

describe('automatic install selection', () => {
    const drivers = [
        { id: 'gpu', status: 'update' },
        { id: 'audio', status: 'current' },
    ] as Driver[];

    it('returns nothing unless auto-install is enabled', () => {
        expect(autoInstallIds(settings(), drivers)).toEqual([]);
    });

    it('selects only available updates', () => {
        expect(autoInstallIds(settings({ autoInstall: true }), drivers)).toEqual(['gpu']);
    });
});
