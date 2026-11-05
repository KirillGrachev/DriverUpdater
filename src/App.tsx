import { lazy, Suspense, useEffect, useLayoutEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { BottomDock } from './components/BottomDock';
import { LoadingScreen } from './components/LoadingScreen';
import { ErrorBoundary } from './components/ErrorBoundary';
import { TitleBar } from './components/TitleBar';
import { ModalHost, ToastHost } from './components/overlays';
import { Button } from './components/ui';
import { HomePage } from './pages/HomePage';

const AboutPage = lazy(() => import('./pages/AboutPage').then((module) => ({ default: module.AboutPage })));
const BackupsPage = lazy(() => import('./pages/BackupsPage').then((module) => ({ default: module.BackupsPage })));
const ComponentsPage = lazy(() =>
    import('./pages/ComponentsPage').then((module) => ({ default: module.ComponentsPage })),
);
const DriversPage = lazy(() => import('./pages/DriversPage').then((module) => ({ default: module.DriversPage })));
const SettingsPage = lazy(() => import('./pages/SettingsPage').then((module) => ({ default: module.SettingsPage })));
import { useApp } from './store/app';

function RouteFallback() {
    const { beginLoading, endLoading } = useApp();
    useLayoutEffect(() => {
        beginLoading();
        return endLoading;
    }, [beginLoading, endLoading]);
    return null;
}

function RouteTransition() {
    const { route, beginLoading, endLoading } = useApp();
    useLayoutEffect(() => {
        beginLoading();
        const timer = window.setTimeout(endLoading, 180);
        return () => {
            window.clearTimeout(timer);
            endLoading();
        };
    }, [beginLoading, endLoading, route]);
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
    const { booting, bootError, loadingCount, route, scanning, t, data, pushToast } = useApp();
    const scanningWithoutData = scanning && !data?.summary.lastCheck;
    const loading = loadingCount > 0 || scanningWithoutData;

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
                // The close guard still works without the toast listener.
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

    if (booting) {
        return (
            <div className="app-window boot-window">
                <LoadingScreen visible />
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
                        {import.meta.env.DEV && bootError ? <pre className="boot-error-detail">{bootError}</pre> : null}
                        <Button variant="primary" onClick={() => window.location.reload()}>
                            {t('boot.retry')}
                        </Button>
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="app-window">
            <RouteTransition />
            <TitleBar />
            <div className="app-body">
                <main className="app-main">
                    <LoadingScreen visible={loading} />
                    <div className={`app-content ${loading ? 'is-loading' : ''}`}>
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
        </div>
    );
}
