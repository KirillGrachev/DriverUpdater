import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';
import settingsFixture from '../../shared/fixtures/settings.json';
import { translate, type TranslateParams } from '../i18n';
import { createBackend, type Backend } from '../ipc/backend';
import type { Driver, InitialData, Settings, SystemInfo } from '../ipc/types';

export type Route = 'home' | 'drivers' | 'updates' | 'backups' | 'settings' | 'about';

export interface Toast {
  id: number;
  text: string;
  tone: 'success' | 'info' | 'error';
}

export interface ConfirmModal {
  title: string;
  text: string;
  confirmLabel: string;
  cancelLabel: string;
  tone: 'primary' | 'danger';
  onConfirm: () => Promise<void> | void;
}

interface AppState {
  route: Route;
  data: InitialData | null;
  systemInfo: SystemInfo | null;
  booting: boolean;
  scanning: boolean;
  installingIds: string[];
  busyRestorePoints: string[];
  toasts: Toast[];
  modal: ConfirmModal | null;
}

interface AppContextValue extends AppState {
  t: (key: string, params?: TranslateParams & { count?: number }) => string;
  locale: string;
  navigate: (route: Route) => void;
  scan: () => Promise<void>;
  installDriver: (id: string) => Promise<void>;
  installUpdates: (ids: string[]) => Promise<void>;
  createRestorePoint: () => Promise<void>;
  requestDeleteRestorePoint: (id: string, name: string) => void;
  requestRestorePoint: (id: string, name: string) => void;
  saveSettings: (settings: Settings) => Promise<void>;
  resetSettings: () => Promise<void>;
  pushToast: (text: string, tone?: Toast['tone']) => void;
  dismissToast: (id: number) => void;
  openModal: (modal: ConfirmModal) => void;
  closeModal: () => void;
}

const AppContext = createContext<AppContextValue | null>(null);

let toastSeq = 0;

export function AppProvider({ children }: { children: ReactNode }) {
  const backendRef = useRef<Backend | null>(null);
  const [state, setState] = useState<AppState>({
    route: 'home',
    data: null,
    systemInfo: null,
    booting: true,
    scanning: false,
    installingIds: [],
    busyRestorePoints: [],
    toasts: [],
    modal: null,
  });

  const patch = useCallback((partial: Partial<AppState>) => {
    setState((prev) => ({ ...prev, ...partial }));
  }, []);

  const pushToast = useCallback((text: string, tone: Toast['tone'] = 'info') => {
    const id = ++toastSeq;
    setState((prev) => ({ ...prev, toasts: [...prev.toasts, { id, text, tone }] }));
    window.setTimeout(() => {
      setState((prev) => ({ ...prev, toasts: prev.toasts.filter((t) => t.id !== id) }));
    }, 4200);
  }, []);

  const dismissToast = useCallback((id: number) => {
    setState((prev) => ({ ...prev, toasts: prev.toasts.filter((t) => t.id !== id) }));
  }, []);

  const openModal = useCallback((modal: ConfirmModal) => patch({ modal }), [patch]);
  const closeModal = useCallback(() => patch({ modal: null }), [patch]);

  // ---- boot -------------------------------------------------------------
  useEffect(() => {
    let cancelled = false;
    (async () => {
      const backend = await createBackend();
      backendRef.current = backend;
      const data = await backend.getInitialData();
      if (cancelled) return;
      patch({ data, systemInfo: data.systemInfo, booting: false });
    })();
    return () => {
      cancelled = true;
    };
  }, [patch]);

  const locale = state.data?.settings.language ?? 'ru';
  const t = useCallback(
    (key: string, params?: TranslateParams & { count?: number }) =>
      translate(locale, key, params),
    [locale],
  );

  // ---- theme ------------------------------------------------------------
  const theme = state.data?.settings.theme ?? 'light';
  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const apply = () => {
      const resolved = theme === 'system' ? (media.matches ? 'dark' : 'light') : theme;
      document.documentElement.dataset.theme = resolved;
    };
    apply();
    if (theme !== 'system') return undefined;
    media.addEventListener('change', apply);
    return () => media.removeEventListener('change', apply);
  }, [theme]);

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  // ---- helpers ----------------------------------------------------------
  const refreshSummary = useCallback(async (drivers: Driver[]) => {
    const backend = backendRef.current;
    if (!backend) return;
    const summary = await backend.computeSummary(drivers);
    setState((prev) => (prev.data ? { ...prev, data: { ...prev.data, drivers, summary } } : prev));
  }, []);

  const navigate = useCallback((route: Route) => patch({ route }), [patch]);

  const scan = useCallback(async () => {
    const backend = backendRef.current;
    if (!backend || state.scanning) return;
    patch({ scanning: true });
    try {
      const result = await backend.checkUpdates();
      setState((prev) =>
        prev.data
          ? {
              ...prev,
              scanning: false,
              data: { ...prev.data, drivers: result.drivers, summary: result.summary },
            }
          : { ...prev, scanning: false },
      );
      const count = result.summary.updatesAvailable;
      pushToast(
        count > 0
          ? translate(locale, 'toasts.scanFound', { count })
          : translate(locale, 'toasts.scanZero'),
        count > 0 ? 'info' : 'success',
      );
    } finally {
      patch({ scanning: false });
    }
  }, [state.scanning, patch, pushToast, locale]);

  const installDriver = useCallback(
    async (id: string) => {
      const backend = backendRef.current;
      if (!backend) return;
      setState((prev) => ({ ...prev, installingIds: [...prev.installingIds, id] }));
      try {
        const updated = await backend.installDriver(id);
        setState((prev) => {
          const drivers = (prev.data?.drivers ?? []).map((d) => (d.id === id ? updated : d));
          return prev.data ? { ...prev, data: { ...prev.data, drivers } } : prev;
        });
        const drivers = state.data?.drivers.map((d) => (d.id === id ? updated : d)) ?? [];
        await refreshSummary(drivers);
        pushToast(translate(locale, 'toasts.driverUpdated', { name: updated.name }), 'success');
      } finally {
        setState((prev) => ({
          ...prev,
          installingIds: prev.installingIds.filter((x) => x !== id),
        }));
      }
    },
    [state.data, refreshSummary, pushToast, locale],
  );

  const installUpdates = useCallback(
    async (ids: string[]) => {
      const backend = backendRef.current;
      if (!backend || ids.length === 0) return;
      setState((prev) => ({
        ...prev,
        installingIds: [...prev.installingIds, ...ids],
      }));
      try {
        const updatedList = await backend.installUpdates(ids);
        const updatedById = new Map(updatedList.map((d) => [d.id, d]));
        setState((prev) => {
          const drivers = (prev.data?.drivers ?? []).map((d) => updatedById.get(d.id) ?? d);
          return prev.data ? { ...prev, data: { ...prev.data, drivers } } : prev;
        });
        const drivers =
          state.data?.drivers.map((d) => updatedById.get(d.id) ?? d) ?? [];
        await refreshSummary(drivers);
        pushToast(translate(locale, 'updates.doneToast', { count: ids.length }), 'success');
      } finally {
        setState((prev) => ({
          ...prev,
          installingIds: prev.installingIds.filter((x) => !ids.includes(x)),
        }));
      }
    },
    [state.data, refreshSummary, pushToast, locale],
  );

  const createRestorePoint = useCallback(async () => {
    const backend = backendRef.current;
    if (!backend) return;
    const name = {
      ru: translate('ru', 'backups.newName'),
      en: translate('en', 'backups.newName'),
    };
    const point = await backend.createRestorePoint(name);
    setState((prev) =>
      prev.data
        ? {
            ...prev,
            data: {
              ...prev.data,
              restorePoints: [point, ...prev.data.restorePoints],
            },
          }
        : prev,
    );
    pushToast(translate(locale, 'backups.createdToast'), 'success');
  }, [pushToast, locale]);

  const withBusyPoint = useCallback(
    async (id: string, fn: () => Promise<void>) => {
      setState((prev) => ({ ...prev, busyRestorePoints: [...prev.busyRestorePoints, id] }));
      try {
        await fn();
      } finally {
        setState((prev) => ({
          ...prev,
          busyRestorePoints: prev.busyRestorePoints.filter((x) => x !== id),
        }));
      }
    },
    [],
  );

  const requestDeleteRestorePoint = useCallback(
    (id: string, name: string) => {
      openModal({
        title: translate(locale, 'backups.confirmDeleteTitle'),
        text: translate(locale, 'backups.confirmDeleteText', { name }),
        confirmLabel: translate(locale, 'backups.confirmOk'),
        cancelLabel: translate(locale, 'backups.confirmCancel'),
        tone: 'danger',
        onConfirm: async () => {
          const backend = backendRef.current;
          if (!backend) return;
          closeModal();
          await withBusyPoint(id, async () => {
            await backend.deleteRestorePoint(id);
            setState((prev) =>
              prev.data
                ? {
                    ...prev,
                    data: {
                      ...prev.data,
                      restorePoints: prev.data.restorePoints.filter((p) => p.id !== id),
                    },
                  }
                : prev,
            );
            pushToast(translate(locale, 'backups.deletedToast'), 'info');
          });
        },
      });
    },
    [openModal, closeModal, withBusyPoint, pushToast, locale],
  );

  const requestRestorePoint = useCallback(
    (id: string, name: string) => {
      openModal({
        title: translate(locale, 'backups.confirmRestoreTitle'),
        text: translate(locale, 'backups.confirmRestoreText', { name }),
        confirmLabel: translate(locale, 'backups.confirmOk'),
        cancelLabel: translate(locale, 'backups.confirmCancel'),
        tone: 'primary',
        onConfirm: async () => {
          const backend = backendRef.current;
          if (!backend) return;
          closeModal();
          await withBusyPoint(id, async () => {
            await backend.restoreFromPoint(id);
            pushToast(translate(locale, 'backups.restoredToast', { name }), 'success');
          });
        },
      });
    },
    [openModal, closeModal, withBusyPoint, pushToast, locale],
  );

  const saveSettings = useCallback(
    async (settings: Settings) => {
      const backend = backendRef.current;
      if (!backend) return;
      const saved = await backend.saveSettings(settings);
      setState((prev) =>
        prev.data ? { ...prev, data: { ...prev.data, settings: saved } } : prev,
      );
      pushToast(translate(locale, 'settings.savedToast'), 'success');
    },
    [pushToast, locale],
  );

  const resetSettings = useCallback(async () => {
    const defaults = JSON.parse(JSON.stringify(settingsFixture)) as Settings;
    await saveSettings(defaults);
    pushToast(translate(locale, 'settings.resetToast'), 'info');
  }, [saveSettings, pushToast, locale]);

  const value = useMemo<AppContextValue>(
    () => ({
      ...state,
      t,
      locale,
      navigate,
      scan,
      installDriver,
      installUpdates,
      createRestorePoint,
      requestDeleteRestorePoint,
      requestRestorePoint,
      saveSettings,
      resetSettings,
      pushToast,
      dismissToast,
      openModal,
      closeModal,
    }),
    [
      state,
      t,
      locale,
      navigate,
      scan,
      installDriver,
      installUpdates,
      createRestorePoint,
      requestDeleteRestorePoint,
      requestRestorePoint,
      saveSettings,
      resetSettings,
      pushToast,
      dismissToast,
      openModal,
      closeModal,
    ],
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useApp(): AppContextValue {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useApp must be used inside <AppProvider>');
  return ctx;
}
