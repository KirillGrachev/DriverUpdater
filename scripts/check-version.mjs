import { readFileSync } from 'node:fs';

const packageVersion = JSON.parse(readFileSync(new URL('../package.json', import.meta.url), 'utf8')).version;
const tauri = JSON.parse(readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8'));
const cargo = readFileSync(new URL('../src-tauri/Cargo.toml', import.meta.url), 'utf8');
const cargoVersion = cargo.match(/^version = "([^"]+)"/m)?.[1];

if (!cargoVersion || tauri.version !== packageVersion || cargoVersion !== packageVersion) {
    console.error('Version mismatch:', { packageVersion, tauri: tauri.version, cargo: cargoVersion });
    process.exit(1);
}
console.log(`version ${packageVersion} is consistent`);
