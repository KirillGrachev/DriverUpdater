import type { CheckInterval, Settings } from '../ipc/types';

const HOUR = 60 * 60 * 1000;
const DAY = 24 * HOUR;
const WEEK = 7 * DAY;

/**
 * Returns the configured automatic-check period in milliseconds.
 *
 * @param interval user-selected interval
 * @returns interval in milliseconds
 */
export function autoCheckInterval(interval: CheckInterval): number {
    if (interval === 'hourly') return HOUR;
    if (interval === 'weekly') return WEEK;
    return DAY;
}

/**
 * Decides whether an automatic update check is due.
 *
 * <p>A missing or malformed timestamp is treated as due, so a fresh install
 * checks once and then records a real timestamp.</p>
 *
 * @param settings current settings
 * @param lastCheck ISO timestamp of the last successful check
 * @param now current epoch milliseconds
 * @returns true when the automatic scan should run
 */
export function shouldAutoCheck(settings: Settings, lastCheck: string | null | undefined, now = Date.now()): boolean {
    if (!settings.autoCheck) return false;
    if (!lastCheck) return true;
    const checkedAt = Date.parse(lastCheck);
    if (!Number.isFinite(checkedAt)) return true;
    return now - checkedAt >= autoCheckInterval(settings.checkInterval);
}
