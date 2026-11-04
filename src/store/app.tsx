import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { isSupportedLocale, loadLocale, translate, type TranslateParams } from '../i18n';
import { createBackend, type Backend } from '../ipc/backend';
import type { Driver, InitialData, InstalledComponent, Settings, SystemInfo, ThemeMode } from '../ipc/types';

export type Route = 'home' | 'drivers' | 'components' | 'backups' | 'settings' | 'about';

export interface Toast {
    id: number;
    text: string;
    tone: 'success' | 'info' | 'error';
    leaving?: boolean;
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
    bootError: string | null;
    scanning: boolean;
    installingIds: string[];
    busyRestorePoints: string[];
    busyDeletePoints: string[];
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
    createRestorePoint: (options?: { silentSuccess?: boolean }) => Promise<boolean>;
    requestDeleteRestorePoint: (id: string, name: string) => void;
    requestRestorePoint: (id: string, name: string) => void;
    saveSettings: (settings: Settings) => Promise<void>;
    updateSettings: (patch: Partial<Settings>) => Promise<void>;
    applyTheme: (theme: ThemeMode) => Promise<void>;
    applyLanguage: (language: string) => Promise<void>;
    openBackupFolder: () => Promise<void>;
    detectInstalledComponents: () => Promise<InstalledComponent[]>;
    getInstalledComponents: () => Promise<string[]>;
    saveInstalledComponents: (ids: string[]) => Promise<void>;
    pushToast: (text: string, tone?: Toast['tone']) => void;
    dismissToast: (id: number) => void;
    openModal: (modal: ConfirmModal) => void;
    closeModal: () => void;
}

const AppContext = createContext<AppContextValue | null>(null);

let toastSeq = 0;

export function AppProvider({ children }: { children: ReactNode }) {
    const backendRef = useRef<Backend | null>(null);
    const dataRef = useRef<InitialData | null>(null);
    const scanningRef = useRef(false);
    const themeSeqRef = useRef(0);
    const languageSeqRef = useRef(0);
    const detectedComponentsRef = useRef<InstalledComponent[] | null>(null);
    const settingsQueueRef = useRef<Promise<unknown>>(Promise.resolve());
    const [state, setState] = useState<AppState>({
        route: 'home',
        data: null,
        systemInfo: null,
        booting: true,
        bootError: null,
        scanning: false,
        installingIds: [],
        busyRestorePoints: [],
        busyDeletePoints: [],
        toasts: [],
        modal: null,
    });

    useEffect(() => {
        dataRef.current = state.data;
    }, [state.data]);

    const patch = useCallback((partial: Partial<AppState>) => {
        setState((prev) => ({ ...prev, ...partial }));
    }, []);

    const dismissToast = useCallback((id: number) => {
        setState((prev) => {
            if (prev.toasts.some((toast) => toast.id === id && toast.leaving)) return prev;
            return {
                ...prev,
                toasts: prev.toasts.map((toast) => (toast.id === id ? { ...toast, leaving: true } : toast)),
            };
        });
        window.setTimeout(() => {
            setState((prev) => ({ ...prev, toasts: prev.toasts.filter((t) => t.id !== id) }));
        }, 240);
    }, []);

    const pushToast = useCallback(
        (text: string, tone: Toast['tone'] = 'info') => {
            const id = ++toastSeq;
            setState((prev) => ({ ...prev, toasts: [...prev.toasts, { id, text, tone }] }));
            window.setTimeout(() => dismissToast(id), 4200);
        },
        [dismissToast],
    );

    const openModal = useCallback((modal: ConfirmModal) => patch({ modal }), [patch]);
    const closeModal = useCallback(() => patch({ modal: null }), [patch]);

    /** Loads the backend, the active locale and the initial application data. */
    useEffect(() => {
        let cancelled = false;
        (async () => {
            try {
                const backend = await createBackend();
                backendRef.current = backend;
                const data = await backend.getInitialData();
                if (cancelled) return;
                if (isSupportedLocale(data.settings.language)) {
                    await loadLocale(data.settings.language);
                }
                if (cancelled) return;
                dataRef.current = data;
                patch({ data, systemInfo: data.systemInfo, booting: false });
            } catch (error) {
                if (cancelled) return;
                /** Keep the window usable even if the backend or a locale chunk fails. */
                patch({
                    bootError: error instanceof Error ? error.message : String(error),
                    booting: false,
                });
            }
        })();
        return () => {
            cancelled = true;
        };
    }, [patch]);

    const requestedLocale = state.data?.settings.language;
    const locale = requestedLocale && isSupportedLocale(requestedLocale) ? requestedLocale : 'ru';
    const t = useCallback(
        (key: string, params?: TranslateParams & { count?: number }) => translate(locale, key, params),
        [locale],
    );

    /** Resolves and persists the active theme, including OS theme changes. */
    const theme =
        state.data?.settings.theme ?? ((document.documentElement.dataset.theme as ThemeMode | undefined) || 'light');
    useEffect(() => {
        const media = window.matchMedia('(prefers-color-scheme: dark)');
        const apply = () => {
            const resolved = theme === 'system' ? (media.matches ? 'dark' : 'light') : theme;
            document.documentElement.dataset.theme = resolved;
            try {
                localStorage.setItem('driver-updater.theme', theme);
            } catch {
                /** storage unavailable */
            }
        };
        apply();
        if (theme !== 'system') return undefined;
        media.addEventListener('change', apply);
        return () => media.removeEventListener('change', apply);
    }, [theme]);

    useEffect(() => {
        document.documentElement.lang = locale;
    }, [locale]);

    /** Shared state helpers used by the command callbacks below. */
    const refreshSummary = useCallback(async (drivers: Driver[]) => {
        const backend = backendRef.current;
        const current = dataRef.current;
        if (!backend || !current) return;
        const summary = await backend.computeSummary(drivers);
        const next = { ...current, drivers, summary };
        dataRef.current = next;
        setState((prev) => (prev.data ? { ...prev, data: next } : prev));
    }, []);

    const navigate = useCallback((route: Route) => patch({ route }), [patch]);

    /**
     * Settings are written as a whole document, so concurrent theme/language
     * saves could otherwise resolve out of order and revert each other.
     */
    const enqueueSettingsWrite = useCallback(<T,>(write: () => Promise<T>): Promise<T> => {
        const result = settingsQueueRef.current.then(write, write);
        settingsQueueRef.current = result.then(
            () => undefined,
            () => undefined,
        );
        return result;
    }, []);

    const scan = useCallback(async () => {
        const backend = backendRef.current;
        if (!backend || scanningRef.current) return;
        scanningRef.current = true;
        patch({ scanning: true });
        try {
            const result = await backend.checkUpdates();
            const current = dataRef.current;
            if (current) {
                dataRef.current = { ...current, drivers: result.drivers, summary: result.summary };
            }
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
                count > 0 ? translate(locale, 'toasts.scanFound', { count }) : translate(locale, 'toasts.scanZero'),
                count > 0 ? 'info' : 'success',
            );
        } catch {
            pushToast(translate(locale, 'toasts.error'), 'error');
        } finally {
            scanningRef.current = false;
            patch({ scanning: false });
        }
    }, [patch, pushToast, locale]);

    const installDriver = useCallback(
        async (id: string) => {
            const backend = backendRef.current;
            const base = dataRef.current;
            if (!backend || !base) return;
            setState((prev) => ({ ...prev, installingIds: [...prev.installingIds, id] }));
            try {
                const updated = await backend.installDriver(id);
                const latest = dataRef.current ?? base;
                const drivers = latest.drivers.map((d) => (d.id === id ? updated : d));
                dataRef.current = { ...latest, drivers };
                setState((prev) => (prev.data ? { ...prev, data: { ...prev.data, drivers } } : prev));
                await refreshSummary(drivers);
                pushToast(translate(locale, 'toasts.driverUpdated', { name: updated.name }), 'success');
            } catch {
                pushToast(translate(locale, 'toasts.error'), 'error');
            } finally {
                setState((prev) => ({
                    ...prev,
                    installingIds: prev.installingIds.filter((x) => x !== id),
                }));
            }
        },
        [refreshSummary, pushToast, locale],
    );

    const installUpdates = useCallback(
        async (ids: string[]) => {
            const backend = backendRef.current;
            const base = dataRef.current;
            if (!backend || !base || ids.length === 0) return;
            setState((prev) => ({
                ...prev,
                installingIds: [...prev.installingIds, ...ids],
            }));
            try {
                const updatedList = await backend.installUpdates(ids);
                const updatedById = new Map(updatedList.map((d) => [d.id, d]));
                const latest = dataRef.current ?? base;
                const drivers = latest.drivers.map((d) => updatedById.get(d.id) ?? d);
                dataRef.current = { ...latest, drivers };
                setState((prev) => (prev.data ? { ...prev, data: { ...prev.data, drivers } } : prev));
                await refreshSummary(drivers);
                pushToast(translate(locale, 'updates.doneToast', { count: ids.length }), 'success');
            } catch {
                pushToast(translate(locale, 'toasts.error'), 'error');
            } finally {
                setState((prev) => ({
                    ...prev,
                    installingIds: prev.installingIds.filter((x) => !ids.includes(x)),
                }));
            }
        },
        [refreshSummary, pushToast, locale],
    );

    const createRestorePoint = useCallback(
        async (options?: { silentSuccess?: boolean }) => {
            const backend = backendRef.current;
            const base = dataRef.current;
            if (!backend || !base) return false;
            try {
                const name = {
                    ru: translate('ru', 'backups.newName'),
                    en: translate('en', 'backups.newName'),
                };
                const point = await backend.createRestorePoint(name);
                const latest = dataRef.current ?? base;
                const next = { ...latest, restorePoints: [point, ...latest.restorePoints] };
                dataRef.current = next;
                setState((prev) => (prev.data ? { ...prev, data: next } : prev));
                if (!options?.silentSuccess) {
                    pushToast(translate(locale, 'backups.createdToast'), 'success');
                }
                return true;
            } catch {
                pushToast(translate(locale, 'toasts.error'), 'error');
                return false;
            }
        },
        [pushToast, locale],
    );

    const withBusyPoint = useCallback(
        async (id: string, fn: () => Promise<void>, kind: 'restore' | 'delete' = 'restore') => {
            const key = kind === 'delete' ? 'busyDeletePoints' : 'busyRestorePoints';
            setState((prev) => ({ ...prev, [key]: [...prev[key], id] }));
            try {
                await fn();
            } catch {
                pushToast(translate(locale, 'toasts.error'), 'error');
            } finally {
                setState((prev) => ({
                    ...prev,
                    [key]: prev[key].filter((x) => x !== id),
                }));
            }
        },
        [pushToast, locale],
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
                    await withBusyPoint(
                        id,
                        async () => {
                            await backend.deleteRestorePoint(id);
                            const base = dataRef.current;
                            if (base) {
                                const next = {
                                    ...base,
                                    restorePoints: base.restorePoints.filter((p) => p.id !== id),
                                };
                                dataRef.current = next;
                                setState((prev) => (prev.data ? { ...prev, data: next } : prev));
                            }
                            pushToast(translate(locale, 'backups.deletedToast'), 'info');
                        },
                        'delete',
                    );
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

    const applyLanguage = useCallback(
        async (language: string) => {
            const backend = backendRef.current;
            const current = dataRef.current?.settings;
            if (!backend || !current) return;
            const seq = ++languageSeqRef.current;
            const loaded = await loadLocale(language);
            if (!loaded || seq !== languageSeqRef.current) return;
            const optimistic = { ...current, language };
            const ref = dataRef.current;
            if (ref) dataRef.current = { ...ref, settings: optimistic };
            setState((prev) =>
                prev.data ? { ...prev, data: { ...prev.data, settings: { ...prev.data.settings, language } } } : prev,
            );
            try {
                const saved = await enqueueSettingsWrite(() => backend.saveSettings({ ...current, language }));
                if (seq !== languageSeqRef.current) return;
                const latest = dataRef.current;
                if (!latest) return;
                const next = { ...latest, settings: saved };
                dataRef.current = next;
                setState((prev) => (prev.data ? { ...prev, data: next } : prev));
            } catch {
                pushToast(translate(locale, 'toasts.error'), 'error');
            }
        },
        [enqueueSettingsWrite, pushToast, locale],
    );

    const saveSettings = useCallback(
        async (settings: Settings) => {
            const backend = backendRef.current;
            const base = dataRef.current;
            if (!backend || !base) return;
            try {
                const saved = await enqueueSettingsWrite(() => backend.saveSettings(settings));
                const next = { ...(dataRef.current ?? base), settings: saved };
                dataRef.current = next;
                setState((prev) => (prev.data ? { ...prev, data: next } : prev));
            } catch {
                pushToast(translate(locale, 'toasts.error'), 'error');
            }
        },
        [enqueueSettingsWrite, pushToast, locale],
    );

    const updateSettings = useCallback(
        async (patchSettings: Partial<Settings>) => {
            const backend = backendRef.current;
            const current = dataRef.current?.settings;
            if (!backend || !current) return;
            const optimistic = { ...current, ...patchSettings };
            dataRef.current = dataRef.current ? { ...dataRef.current, settings: optimistic } : dataRef.current;
            setState((prev) => (prev.data ? { ...prev, data: { ...prev.data, settings: optimistic } } : prev));
            try {
                const saved = await enqueueSettingsWrite(() => backend.saveSettings(optimistic));
                const latest = dataRef.current;
                if (!latest) return;
                const next = { ...latest, settings: saved };
                dataRef.current = next;
                setState((prev) => (prev.data ? { ...prev, data: next } : prev));
            } catch {
                pushToast(translate(locale, 'toasts.error'), 'error');
            }
        },
        [enqueueSettingsWrite, pushToast, locale],
    );

    const openBackupFolder = useCallback(async () => {
        try {
            await backendRef.current?.openBackupFolder();
        } catch {
            pushToast(translate(locale, 'toasts.error'), 'error');
        }
    }, [pushToast, locale]);

    const detectInstalledComponents = useCallback(async () => {
        if (detectedComponentsRef.current) return detectedComponentsRef.current;
        const backend = backendRef.current;
        if (!backend) return [];
        try {
            const detected = await backend.detectInstalledComponents();
            detectedComponentsRef.current = detected;
            return detected;
        } catch {
            detectedComponentsRef.current = [];
            return [];
        }
    }, []);

    const getInstalledComponents = useCallback(async () => {
        const backend = backendRef.current;
        if (!backend) return [];
        try {
            return await backend.getInstalledComponents();
        } catch {
            return [];
        }
    }, []);

    const saveInstalledComponents = useCallback(async (ids: string[]) => {
        const backend = backendRef.current;
        if (!backend) return;
        try {
            await backend.saveInstalledComponents(ids);
        } catch {
            /* The toast host reports user-facing failures elsewhere. */
        }
    }, []);

    /**
     * Theme is previewed and persisted immediately, so the switch feels alive
     * without having to press "Save" first.
     */
    const applyTheme = useCallback(
        async (theme: ThemeMode) => {
            const backend = backendRef.current;
            const current = dataRef.current?.settings;
            if (!backend || !current) return;
            const seq = ++themeSeqRef.current;
            const optimistic = { ...current, theme };
            dataRef.current = dataRef.current ? { ...dataRef.current, settings: optimistic } : dataRef.current;
            setState((prev) =>
                prev.data ? { ...prev, data: { ...prev.data, settings: { ...prev.data.settings, theme } } } : prev,
            );
            try {
                const saved = await enqueueSettingsWrite(() => backend.saveSettings({ ...current, theme }));
                if (seq !== themeSeqRef.current) return;
                const latest = dataRef.current;
                if (!latest) return;
                const next = { ...latest, settings: saved };
                dataRef.current = next;
                setState((prev) => (prev.data ? { ...prev, data: next } : prev));
            } catch {
                /** optimistic update already applied */
            }
        },
        [enqueueSettingsWrite],
    );

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
            updateSettings,
            applyTheme,
            applyLanguage,
            openBackupFolder,
            detectInstalledComponents,
            getInstalledComponents,
            saveInstalledComponents,
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
            updateSettings,
            applyTheme,
            applyLanguage,
            openBackupFolder,
            detectInstalledComponents,
            getInstalledComponents,
            saveInstalledComponents,
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
