import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const localesDir = resolve(root, 'src/i18n/locales');
const baselinePath = resolve(root, 'shared/locales-baseline.json');
const update = process.argv.includes('--update');

function leaves(value, prefix = '', result = new Set()) {
    if (value && typeof value === 'object' && !Array.isArray(value)) {
        for (const [key, child] of Object.entries(value)) {
            leaves(child, prefix ? `${prefix}.${key}` : key, result);
        }
    } else {
        result.add(prefix);
    }
    return result;
}

const english = JSON.parse(readFileSync(resolve(localesDir, 'en.json'), 'utf8'));
const englishKeys = leaves(english);
const localeFiles = readdirSync(localesDir).filter((name) => name.endsWith('.json') && name !== 'en.json');

const current = {};
for (const file of localeFiles.sort()) {
    const code = file.slice(0, -5);
    const dictionary = JSON.parse(readFileSync(resolve(localesDir, file), 'utf8'));
    const missing = [...englishKeys].filter((key) => !leaves(dictionary).has(key)).sort();
    current[code] = missing;
}

if (update) {
    writeFileSync(baselinePath, `${JSON.stringify(current, null, 2)}\n`, 'utf8');
    console.log(`Wrote ${baselinePath}`);
    process.exit(0);
}

let baseline;
try {
    baseline = JSON.parse(readFileSync(baselinePath, 'utf8'));
} catch {
    console.error('Missing shared/locales-baseline.json. Run: node scripts/check-locales.mjs --update');
    process.exit(1);
}

let failed = false;
for (const [code, missing] of Object.entries(current)) {
    const known = new Set(baseline[code] ?? []);
    const added = missing.filter((key) => !known.has(key));
    if (added.length > 0) {
        failed = true;
        console.error(`${code}: ${added.length} new missing keys`);
        for (const key of added) console.error(`  - ${key}`);
    }
}

if (failed) {
    console.error('Locale coverage regressed. Add the missing translations or refresh the baseline deliberately.');
    process.exit(1);
}

console.log(`Locale coverage checked: ${Object.keys(current).length} locales, no new missing keys.`);
