import { readdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const setupSource = readFileSync(join(root, 'installer/ui/languages.js'), 'utf8');
const setupCodes = [...setupSource.matchAll(/code:\s*'([^']+)'/g)].map((match) => match[1]).sort();
const appCodes = readdirSync(join(root, 'src/i18n/locales'))
    .filter((name) => name.endsWith('.json'))
    .map((name) => name.slice(0, -5))
    .sort();

const missing = appCodes.filter((code) => !setupCodes.includes(code));
const extra = setupCodes.filter((code) => !appCodes.includes(code));
if (missing.length > 0 || extra.length > 0 || new Set(setupCodes).size !== setupCodes.length) {
    console.error('Setup language list is out of sync with src/i18n/locales.');
    if (missing.length > 0) console.error(`Missing: ${missing.join(', ')}`);
    if (extra.length > 0) console.error(`Extra: ${extra.join(', ')}`);
    if (new Set(setupCodes).size !== setupCodes.length) console.error('Duplicate setup language code.');
    process.exit(1);
}
console.log(`Setup languages checked: ${setupCodes.length} codes match the app locales.`);
