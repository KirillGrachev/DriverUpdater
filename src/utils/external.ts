import { isTauriRuntime } from '../ipc/backend';

const EXTERNAL_HTTP = /^https?:\/\//i;

export function isExternalHttpUrl(url: string): boolean {
    return EXTERNAL_HTTP.test(url);
}

/**
 * Opens an external URL in the system browser.
 * A Tauri webview has no tabbed browser, so target="_blank" alone does nothing;
 * links must go through the opener plugin.
 */
export async function openExternal(url: string): Promise<void> {
    if (!isExternalHttpUrl(url)) return;
    if (!isTauriRuntime()) {
        window.open(url, '_blank', 'noopener,noreferrer');
        return;
    }

    /**
     * First try to hand the URL to a browser that is already running; only
     * fall back to the OS default handler when no browser process exists.
     */
    try {
        const { invoke } = await import('@tauri-apps/api/core');
        const opened = await invoke<boolean>('open_external_url', { url });
        if (opened) return;
    } catch {
        /** fall through to the system opener */
    }

    const { openUrl } = await import('@tauri-apps/plugin-opener');
    await openUrl(url);
}
