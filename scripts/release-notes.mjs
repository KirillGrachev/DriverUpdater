#!/usr/bin/env node
/**
 * Builds human-readable GitHub release notes from conventional commits.
 *
 * <p>The release workflow pushes directly to main, so GitHub's generated notes
 * usually collapse to a single "Full Changelog" link. This script groups the
 * commit subjects between two tags and gives every user-facing change a bullet
 * of its own.</p>
 *
 * @param --from previous tag
 * @param --to release tag
 * @param --output markdown file to write
 * @returns process exit code
 */
import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';

const args = Object.fromEntries(
    process.argv.slice(2).reduce((pairs, value, index, values) => {
        if (value.startsWith('--')) {
            pairs.push([value.slice(2), values[index + 1]]);
        }
        return pairs;
    }, []),
);

const from = args.from;
const to = args.to;
const output = args.output;
if (!from || !to || !output) {
    console.error('usage: release-notes.mjs --from <tag> --to <tag> --output <file>');
    process.exit(1);
}

const repository = process.env.GITHUB_REPOSITORY || 'KirillGrachev/DriverUpdater';
const base = `https://github.com/${repository}`;

const raw = execFileSync(
    'git',
    ['log', `${from}..${to}`, '--pretty=format:%H%x1f%s%x1e', '--no-merges'],
    { encoding: 'utf8' },
);
const commits = raw
    .split('\x1e')
    .map((record) => record.trim())
    .filter(Boolean)
    .map((record) => {
        const [hash, subject] = record.split('\x1f');
        return { hash, subject };
    });

const sections = [
    { key: 'features', title: 'Features' },
    { key: 'fixes', title: 'Fixes' },
    { key: 'performance', title: 'Performance' },
    { key: 'docs', title: 'Documentation' },
    { key: 'build', title: 'Build and CI' },
    { key: 'internal', title: 'Internal' },
];
const grouped = Object.fromEntries(sections.map((section) => [section.key, []]));
const uncategorized = [];

function classify(type) {
    switch (type) {
        case 'feat':
            return 'features';
        case 'fix':
            return 'fixes';
        case 'perf':
            return 'performance';
        case 'docs':
            return 'docs';
        case 'build':
        case 'ci':
        case 'test':
            return 'build';
        case 'chore':
        case 'refactor':
        case 'style':
        case 'revert':
            return 'internal';
        default:
            return null;
    }
}

for (const commit of commits) {
    const subject = commit.subject.trim();
    if (/^Merge\b/i.test(subject) || /^chore(\([^)]*\))?:\s*release\b/i.test(subject)) {
        continue;
    }
    const match = subject.match(/^([a-z]+)(?:\(([^)]+)\))?(!)?:\s*(.+)$/i);
    if (!match) {
        uncategorized.push(commit);
        continue;
    }
    const [, type, scope, breaking, message] = match;
    const section = classify(type.toLowerCase());
    if (!section) {
        uncategorized.push(commit);
        continue;
    }
    const scopeLabel = scope
        ? `**${scope.replace(/[-_]/g, ' ').replace(/\b\w/g, (letter) => letter.toUpperCase())}:** `
        : '';
    const breakingLabel = breaking ? '**Breaking:** ' : '';
    const text = message.charAt(0).toUpperCase() + message.slice(1);
    grouped[section].push(`${breakingLabel}${scopeLabel}${text} ([${commit.hash.slice(0, 7)}](${base}/commit/${commit.hash}))`);
}

const lines = ['## What changed', ''];
for (const section of sections) {
    if (grouped[section.key].length === 0) continue;
    lines.push(`### ${section.title}`, '');
    for (const item of grouped[section.key]) lines.push(`- ${item}`);
    lines.push('');
}
if (uncategorized.length > 0) {
    lines.push('### Changes', '');
    for (const commit of uncategorized) {
        lines.push(`- ${commit.subject.trim()} ([${commit.hash.slice(0, 7)}](${base}/commit/${commit.hash}))`);
    }
    lines.push('');
}
lines.push(`**Full Changelog**: ${base}/compare/${from}...${to}`, '');

writeFileSync(output, lines.join('\n'), 'utf8');
console.log(`wrote ${output} (${commits.length} commits)`);
