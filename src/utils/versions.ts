/**
 * Semantic version comparison tolerant to mixed numeric segments
 * (e.g. "572.16" vs "576.02", "10.68.123.2025" vs "10.68.123.2025").
 */
export function compareVersions(a: string, b: string): number {
    const pa = a.split('.');
    const pb = b.split('.');
    const len = Math.max(pa.length, pb.length);
    for (let i = 0; i < len; i += 1) {
        const na = Number.parseInt(pa[i] ?? '0', 10);
        const nb = Number.parseInt(pb[i] ?? '0', 10);
        const sa = Number.isNaN(na) ? 0 : na;
        const sb = Number.isNaN(nb) ? 0 : nb;
        if (sa !== sb) return sa < sb ? -1 : 1;
    }
    return 0;
}

export function isNewerVersion(candidate: string, current: string): boolean {
    return compareVersions(candidate, current) > 0;
}
