import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { AppProvider } from './store/app';
import { isTauriRuntime } from './ipc/backend';
import { isExternalHttpUrl, openExternal } from './utils/external';
import './styles/tokens.css';
import './styles/base.css';
import './styles/components.css';
import './styles/pages.css';

function isEditable(target: EventTarget | null): boolean {
    return (
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        (target instanceof HTMLElement && target.isContentEditable)
    );
}

/**
 * Remove every browser/webview affordance that leaks the web origin:
 * context menu, reload/print/save/devtools/zoom shortcuts, history navigation,
 * drag-and-drop of files/images and Ctrl+wheel zoom.
 */
function installDesktopGuards(): void {
    document.addEventListener('contextmenu', (event) => event.preventDefault());
    document.addEventListener('dragstart', (event) => event.preventDefault());
    document.addEventListener('dragover', (event) => event.preventDefault());
    document.addEventListener('drop', (event) => event.preventDefault());

    document.addEventListener(
        'wheel',
        (event) => {
            if (event.ctrlKey) event.preventDefault();
        },
        { passive: false },
    );

    document.addEventListener('keydown', (event) => {
        const key = event.key.toLowerCase();
        const ctrl = event.ctrlKey || event.metaKey;
        const browserShortcut =
            event.key === 'F5' ||
            event.key === 'F11' ||
            event.key === 'F12' ||
            event.key === 'ContextMenu' ||
            (event.shiftKey && event.key === 'F10') ||
            (ctrl &&
                ['r', 'p', 's', 'u', 'f', 'g', 'h', 'j', 'd', 'l', 'n', 't', 'o', '+', '-', '0', '='].includes(key)) ||
            (ctrl && event.shiftKey && ['i', 'j', 'c', 'r', 'delete'].includes(key)) ||
            (event.altKey && (key === 'arrowleft' || key === 'arrowright')) ||
            (key === 'backspace' && !isEditable(event.target));

        if (browserShortcut) {
            event.preventDefault();
            event.stopPropagation();
        }
    });

    document.addEventListener('focusin', (event) => {
        if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
            event.target.autocomplete = 'off';
            event.target.spellcheck = false;
        }
    });
}

installDesktopGuards();

/**
 * A Tauri webview has no browser tabs: every external <a> must be routed
 */
/**
 * through the opener plugin instead of target="_blank".
 */
if (isTauriRuntime()) {
    document.addEventListener('click', (event) => {
        if (event.defaultPrevented || event.button !== 0) return;
        if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
        const target = event.target;
        if (!(target instanceof Element)) return;
        const anchor = target.closest('a[href]');
        if (!(anchor instanceof HTMLAnchorElement)) return;
        if (!isExternalHttpUrl(anchor.href)) return;
        event.preventDefault();
        void openExternal(anchor.href).catch(() => undefined);
    });
}

/**
 * Keeps keyboard focus rings available while suppressing them after a mouse
 * or touch interaction, including WebView2 builds that keep :focus-visible.
 */
function trackInputModality() {
    const root = document.documentElement;
    root.dataset.input = 'pointer';
    window.addEventListener(
        'pointerdown',
        () => {
            root.dataset.input = 'pointer';
        },
        { capture: true, passive: true },
    );
    window.addEventListener(
        'keydown',
        () => {
            root.dataset.input = 'keyboard';
        },
        { capture: true },
    );
}

trackInputModality();

createRoot(document.getElementById('root')!).render(
    <StrictMode>
        <AppProvider>
            <App />
        </AppProvider>
    </StrictMode>,
);
