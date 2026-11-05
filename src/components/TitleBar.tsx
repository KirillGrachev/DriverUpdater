import { useCallback, useEffect, useState } from 'react';
import logo from '../assets/branding/logo.png';
import { isTauriRuntime } from '../ipc/backend';
import { useApp } from '../store/app';
import { IconClose, IconMaximize, IconMinimize } from './icons';

interface WindowApi {
    minimize: () => Promise<void>;
    toggleMaximize: () => Promise<void>;
    close: () => Promise<void>;
    isMaximized: () => Promise<boolean>;
}

async function loadWindowApi(): Promise<WindowApi | null> {
    if (!isTauriRuntime()) return null;
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    const win = getCurrentWindow();
    return {
        minimize: () => win.minimize(),
        toggleMaximize: () => win.toggleMaximize(),
        close: () => win.close(),
        isMaximized: () => win.isMaximized(),
    };
}

/**
 * Custom title bar for the frameless Tauri window.
 * In browser preview mode the buttons explain themselves instead of acting.
 */
export function TitleBar() {
    const { t } = useApp();
    const [api, setApi] = useState<WindowApi | null>(null);
    const [maximized, setMaximized] = useState(false);

    useEffect(() => {
        let cancelled = false;
        void loadWindowApi()
            .then((loaded) => {
                if (!cancelled) setApi(loaded);
            })
            .catch(() => {
                if (!cancelled) setApi(null);
            });
        return () => {
            cancelled = true;
        };
    }, []);

    const refreshMaximized = useCallback(async () => {
        if (!api) return;
        try {
            setMaximized(await api.isMaximized());
        } catch {
            setMaximized(false);
        }
    }, [api]);

    useEffect(() => {
        let cancelled = false;
        if (!api) return undefined;
        void api
            .isMaximized()
            .then((value) => {
                if (!cancelled) setMaximized(value);
            })
            .catch(() => {
                if (!cancelled) setMaximized(false);
            });
        return () => {
            cancelled = true;
        };
    }, [api]);

    useEffect(() => {
        const onChange = () => void refreshMaximized();
        window.addEventListener('resize', onChange);
        return () => window.removeEventListener('resize', onChange);
    }, [refreshMaximized]);

    const desktopOnly = api === null ? t('titlebar.desktopOnly') : undefined;

    return (
        <header className="titlebar" data-tauri-drag-region>
            <div className="titlebar-brand" data-tauri-drag-region>
                <img src={logo} alt="" aria-hidden="true" draggable={false} />
                <span>Driver Updater</span>
            </div>
            <div className="titlebar-spacer" data-tauri-drag-region />
            <div className="titlebar-controls">
                <button
                    type="button"
                    className="titlebar-btn"
                    title={desktopOnly ?? t('titlebar.minimize')}
                    aria-label={t('titlebar.minimize')}
                    onClick={() => {
                        void api?.minimize().catch(() => undefined);
                    }}
                >
                    <IconMinimize size={13} strokeWidth={1.6} />
                </button>
                <button
                    type="button"
                    className="titlebar-btn"
                    title={desktopOnly ?? (maximized ? t('titlebar.restore') : t('titlebar.maximize'))}
                    aria-label={t('titlebar.maximize')}
                    onClick={() => {
                        void api
                            ?.toggleMaximize()
                            .then(refreshMaximized)
                            .catch(() => undefined);
                    }}
                >
                    <IconMaximize size={12} strokeWidth={1.6} />
                </button>
                <button
                    type="button"
                    className="titlebar-btn titlebar-close"
                    title={desktopOnly ?? t('titlebar.close')}
                    aria-label={t('titlebar.close')}
                    onClick={() => void api?.close().catch(() => undefined)}
                >
                    <IconClose size={13} strokeWidth={1.6} />
                </button>
            </div>
        </header>
    );
}
