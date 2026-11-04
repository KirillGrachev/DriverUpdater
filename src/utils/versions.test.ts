import { describe, expect, it } from 'vitest';
import { compareVersions, isNewerVersion } from './versions';

describe('compareVersions', () => {
  it('compares simple numeric versions', () => {
    expect(compareVersions('572.16', '576.02')).toBeLessThan(0);
    expect(compareVersions('576.02', '572.16')).toBeGreaterThan(0);
    expect(compareVersions('1.0.0', '1.0.0')).toBe(0);
  });

  it('handles different segment counts', () => {
    expect(compareVersions('10.1.28.6', '10.1.36.7')).toBeLessThan(0);
    expect(compareVersions('6.0.9805.1', '6.0.9803.1')).toBeGreaterThan(0);
    expect(compareVersions('2.0', '2.0.0')).toBe(0);
  });

  it('treats non-numeric segments as zero', () => {
    expect(compareVersions('1.0.beta', '1.0.0')).toBe(0);
  });

  it('isNewerVersion reflects ordering', () => {
    expect(isNewerVersion('31.0.101.5793', '31.0.101.5522')).toBe(true);
    expect(isNewerVersion('10.0.26100.4202', '10.0.26100.4301')).toBe(false);
  });
});
