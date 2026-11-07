const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke;
const listen = tauri?.event?.listen;

const languages = Array.isArray(window.SETUP_LANGUAGES) ? window.SETUP_LANGUAGES : [];

const state = {
    screen: 'welcome',
    language: 'en',
    theme: 'light',
    installPath: 'C:\\Program Files\\Driver Updater',
    launching: true,
};

const screens = [...document.querySelectorAll('[data-screen]')];
const languageList = document.getElementById('language-list');
const languageSearch = document.getElementById('language-search');
const themeGrid = document.getElementById('theme-grid');
const installPath = document.getElementById('install-path');
const progressBar = document.getElementById('progress-bar');
const progressValue = document.getElementById('progress-value');
const installStage = document.getElementById('install-stage');
const installError = document.getElementById('install-error');
const locationError = document.getElementById('location-error');
const installButton = document.getElementById('install');

function renderDots() {
    const active = { welcome: 0, language: 1, theme: 2, location: 2, installing: 2, complete: 2 }[state.screen] ?? 0;
    for (const group of document.querySelectorAll('.dots')) {
        group.innerHTML = '';
        for (let index = 0; index < 3; index += 1) {
            const dot = document.createElement('i');
            if (index === active) dot.classList.add('is-active');
            group.append(dot);
        }
    }
}

function showScreen(name) {
    state.screen = name;
    for (const screen of screens) screen.classList.toggle('is-active', screen.dataset.screen === name);
    renderDots();
}

function renderLanguages() {
    const query = languageSearch.value.trim().toLowerCase();
    languageList.innerHTML = '';
    for (const language of languages) {
        if (query && !language.label.toLowerCase().includes(query) && !language.code.includes(query)) continue;
        const row = document.createElement('button');
        row.type = 'button';
        row.className = `language-row${state.language === language.code ? ' is-selected' : ''}`;
        row.innerHTML = `
            <img
                class="flag"
                src="assets/flags/${language.code}.svg"
                width="26"
                height="19"
                alt=""
                aria-hidden="true"
                draggable="false"
            />
            <span class="language-name">${language.label}</span>
            <span class="radio" aria-hidden="true"></span>
        `;
        row.addEventListener('click', () => {
            state.language = language.code;
            renderLanguages();
        });
        languageList.append(row);
    }
}

function renderThemes() {
    for (const card of themeGrid.querySelectorAll('.theme-card')) {
        card.classList.toggle('is-selected', card.dataset.theme === state.theme);
    }
}

const requiredSpace = document.getElementById('required-space');
const availableSpace = document.getElementById('available-space');

function formatBytes(bytes) {
    if (!Number.isFinite(bytes) || bytes <= 0) return '—';
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1) return `${gb.toFixed(1)} GB`;
    return `${Math.max(1, Math.round(bytes / (1024 * 1024)))} MB`;
}

async function refreshSpace() {
    if (!invoke) return;
    try {
        const space = await invoke('installation_space', { path: state.installPath });
        requiredSpace.textContent = formatBytes(space.requiredBytes);
        availableSpace.textContent = formatBytes(space.availableBytes);
    } catch {
        requiredSpace.textContent = '—';
        availableSpace.textContent = '—';
    }
}

function setProgress(percent, stage) {
    const value = Math.max(0, Math.min(100, Math.round(percent)));
    progressBar.style.width = `${value}%`;
    progressValue.textContent = `${value}%`;
    if (stage) installStage.textContent = stage;
}

function setBusy(busy) {
    installButton.disabled = busy;
    for (const button of document.querySelectorAll('[data-back], [data-next]')) button.disabled = busy;
}

document.addEventListener('click', async (event) => {
    const target = event.target.closest('button');
    if (!target) return;
    if (target.dataset.action === 'close') {
        if (invoke) await invoke('close_setup');
        else window.close();
        return;
    }
    if (target.dataset.action === 'minimize') {
        if (invoke) await invoke('minimize_setup');
        return;
    }
    if (target.dataset.action === 'maximize') {
        if (invoke) await invoke('toggle_maximize');
        return;
    }
    if (target.dataset.next) {
        showScreen(target.dataset.next);
        if (target.dataset.next === 'location') void refreshSpace();
        return;
    }
    if (target.dataset.back) {
        showScreen(target.dataset.back);
        return;
    }
    if (target.dataset.theme) {
        state.theme = target.dataset.theme;
        renderThemes();
        return;
    }
});

languageSearch.addEventListener('input', renderLanguages);
installPath.addEventListener('input', () => {
    state.installPath = installPath.value;
    void refreshSpace();
});

document.getElementById('browse').addEventListener('click', async () => {
    if (!invoke) return;
    const selected = await invoke('choose_directory');
    if (selected) {
        state.installPath = selected;
        installPath.value = selected;
        void refreshSpace();
    }
});

document.getElementById('desktop-shortcut').addEventListener('change', (event) => {
    state.desktopShortcut = event.target.checked;
});

document.getElementById('install-retry').addEventListener('click', () => {
    installError.textContent = '';
    document.getElementById('install-retry').hidden = true;
    showScreen('location');
});

installButton.addEventListener('click', async () => {
    setBusy(true);
    installError.textContent = '';
    locationError.textContent = '';
    document.getElementById('install-retry').hidden = true;
    setProgress(2, 'Preparing installation files...');
    showScreen('installing');

    if (!invoke) {
        setProgress(72, 'Preparing installation files...');
        installError.textContent = 'Preview mode: the Tauri backend is not available.';
        setBusy(false);
        return;
    }

    let unlisten;
    if (listen) {
        listen('install-progress', (event) => {
            const payload = event.payload ?? {};
            setProgress(payload.percent ?? 0, payload.stage);
        })
            .then((stop) => {
                unlisten = stop;
            })
            .catch(() => {
                // Progress events are optional; the final result still matters.
            });
    }

    try {
        await invoke('install', {
            request: {
                installPath: state.installPath,
                language: state.language,
                theme: state.theme,
                desktopShortcut: document.getElementById('desktop-shortcut').checked,
            },
        });
        setProgress(100, 'Finishing...');
        await new Promise((resolve) => setTimeout(resolve, 350));
        showScreen('complete');
    } catch (error) {
        installError.textContent = String(error);
        document.getElementById('install-retry').hidden = false;
        showScreen('installing');
    } finally {
        unlisten?.();
        setBusy(false);
    }
});

document.getElementById('finish').addEventListener('click', async () => {
    state.launching = document.getElementById('launch-app').checked;
    if (!invoke) {
        window.close();
        return;
    }
    await invoke('finish_setup', { launch: state.launching });
});

renderLanguages();
renderThemes();
renderDots();
void refreshSpace();
