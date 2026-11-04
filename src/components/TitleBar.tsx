import { useCallback, useEffect, useState } from 'react';
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
    void loadWindowApi().then((loaded) => {
      if (!cancelled) setApi(loaded);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const refreshMaximized = useCallback(async () => {
    if (api) setMaximized(await api.isMaximized());
  }, [api]);

  useEffect(() => {
    void refreshMaximized();
  }, [refreshMaximized]);

  const desktopOnly = api === null ? t('titlebar.desktopOnly') : undefined;

  return (
    <header className="titlebar" data-tauri-drag-region>
      <div className="titlebar-spacer" data-tauri-drag-region />
      <div className="titlebar-controls">
        <button
          type="button"
          className="titlebar-btn"
          title={desktopOnly ?? t('titlebar.minimize')}
          aria-label={t('titlebar.minimize')}
          onClick={() => void api?.minimize()}
        >
          <IconMinimize size={16} />
        </button>
        <button
          type="button"
          className="titlebar-btn"
          title={desktopOnly ?? (maximized ? t('titlebar.restore') : t('titlebar.maximize'))}
          aria-label={t('titlebar.maximize')}
          onClick={() => {
            void api?.toggleMaximize().then(refreshMaximized);
          }}
        >
          <IconMaximize size={14} />
        </button>
        <button
          type="button"
          className="titlebar-btn titlebar-close"
          title={desktopOnly ?? t('titlebar.close')}
          aria-label={t('titlebar.close')}
          onClick={() => void api?.close()}
        >
          <IconClose size={16} />
        </button>
      </div>
    </header>
  );
}
