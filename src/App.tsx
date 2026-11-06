import { lazy, Suspense, useEffect, useLayoutEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { BottomDock } from './components/BottomDock';
import { LoadingScreen, useDelayedLoading } from './components/LoadingScreen';
import { ErrorBoundary } from './components/ErrorBoundary';
import { TitleBar } from './components/TitleBar';
import { ModalHost, ToastHost } from './components/overlays';
import { Button } from './components/ui';
import { HomePage } from './pages/HomePage';

const loadAboutPage = () => import('./pages/AboutPage').then((module) => ({ default: module.AboutPage }));
const loadBackupsPage = () => import('./pages/BackupsPage').then((module) => ({ default: module.BackupsPage }));
const loadComponentsPage = () =>
    import('./pages/ComponentsPage').then((module) => ({ default: module.ComponentsPage }));
const loadDriversPage = () => import('./pages/DriversPage').then((module) => ({ default: module.DriversPage }));
const loadSettingsPage = () => import('./pages/SettingsPage').then((module) => ({ default: module.SettingsPage }));

const AboutPage = lazy(loadAboutPage);
const BackupsPage = lazy(loadBackupsPage);
const ComponentsPage = lazy(loadComponentsPage);
const DriversPage = lazy(loadDriversPage);
const SettingsPage = lazy(loadSettingsPage);

const PAGE_LOADERS = [loadAboutPage, loadBackupsPage, loadComponentsPage, loadDriversPage, loadSettingsPage];

import { useApp } from './store/app';

function RouteFallback() {
    const { t } = useApp();
    return <LoadingScreen visible label={t('components.loading')} />;
}

function RouteScrollReset() {
    const { route } = useApp();
    useLayoutEffect(() => {
        document.querySelector('.app-main')?.scrollTo({ top: 0, left: 0, behavior: 'auto' });
    }, [route]);
    return null;
}

function CurrentPage() {
    const { route } = useApp();
    switch (route) {
        case 'home':
            return <HomePage />;
        case 'drivers':
            return <DriversPage />;
        case 'components':
            return <ComponentsPage />;
        case 'backups':
            return <BackupsPage />;
        case 'settings':
            return <SettingsPage />;
        case 'about':
            return <AboutPage />;
        default:
            return null;
    }
}

export function App() {
    const { booting, bootError, loadingCount, route, t, data, pushToast } = useApp();
    const smoothLoading = useDelayedLoading(loadingCount > 0);
    const smoothBoot = useDelayedLoading(booting, 0, 500);

    useEffect(() => {
        for (const load of PAGE_LOADERS) {
            void load().catch(() => {
                /*
                 * The lazy route still reports the real error when opened.
                 */
            });
        }
    }, []);

    useEffect(() => {
        if (!('__TAURI_INTERNALS__' in window)) return undefined;
        let unlisten: (() => void) | undefined;
        let cancelled = false;
        void import('@tauri-apps/api/event')
            .then(({ listen }) =>
                listen('close-blocked', () => {
                    pushToast(t('components.installing'), 'info');
                }),
            )
            .then((stop) => {
                if (cancelled) stop();
                else unlisten = stop;
            })
            .catch(() => {
                /*
                 * The close guard still works without the toast listener.
                 */
            });
        return () => {
            cancelled = true;
            unlisten?.();
        };
    }, [pushToast, t]);

    useEffect(() => {
        /**
         * The native window starts hidden so the webview never paints a white
         * frame. It is shown on the first React commit, with the themed boot
         * splash already in place while the blocking scan finishes.
         */
        if ('__TAURI_INTERNALS__' in window) {
            void getCurrentWindow().show();
        }
    }, []);

    if (booting || smoothBoot) {
        return (
            <div className="app-window boot-window">
                <LoadingScreen visible label={t('components.loading')} />
            </div>
        );
    }

    if (bootError) {
        return (
            <div className="app-window">
                <TitleBar />
                <div className="app-body boot-failure">
                    <div className="card boot-failure-card">
                        <h1>{t('boot.errorTitle')}</h1>
                        <p>{t('boot.errorSub')}</p>
                        {import.meta.env.DEV ? <pre className="boot-error-detail">{bootError}</pre> : null}
                        <Button variant="primary" onClick={() => window.location.reload()}>
                            {t('boot.retry')}
                        </Button>
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className={`app-window ${smoothLoading ? 'is-loading' : ''}`}>
            <RouteScrollReset />
            <TitleBar />
            <div className="app-body">
                <main className="app-main">
                    <div className={`app-content ${smoothLoading ? 'is-loading' : ''}`}>
                        {data?.scanError ? (
                            <div className="notice-bar notice-error scan-error" role="alert">
                                <div>
                                    <div className="notice-title">{t('common.scanError')}</div>
                                    <div className="notice-sub">{t('common.scanErrorHint')}</div>
                                </div>
                            </div>
                        ) : null}
                        <ErrorBoundary
                            title={t('boot.errorTitle')}
                            message={t('boot.errorSub')}
                            reloadLabel={t('boot.retry')}
                        >
                            <Suspense fallback={<RouteFallback />}>
                                <CurrentPage key={route} />
                            </Suspense>
                        </ErrorBoundary>
                    </div>
                </main>
            </div>
            <BottomDock />
            <ToastHost />
            <ModalHost />
            <LoadingScreen visible={smoothLoading} label={t('components.loading')} />
        </div>
    );
}
