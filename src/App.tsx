import { lazy, Suspense, useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { BottomDock } from './components/BottomDock';
import { TitleBar } from './components/TitleBar';
import { ModalHost, ToastHost } from './components/overlays';
import { Button, Spinner } from './components/ui';
import { HomePage } from './pages/HomePage';

const AboutPage = lazy(() => import('./pages/AboutPage').then((module) => ({ default: module.AboutPage })));
const BackupsPage = lazy(() => import('./pages/BackupsPage').then((module) => ({ default: module.BackupsPage })));
const ComponentsPage = lazy(() =>
    import('./pages/ComponentsPage').then((module) => ({ default: module.ComponentsPage })),
);
const DriversPage = lazy(() => import('./pages/DriversPage').then((module) => ({ default: module.DriversPage })));
const SettingsPage = lazy(() => import('./pages/SettingsPage').then((module) => ({ default: module.SettingsPage })));
import { useApp } from './store/app';

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
    const { booting, bootError, route, t } = useApp();

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
                <div className="boot-splash">
                    <Spinner size={34} />
                </div>
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
            <TitleBar />
            <div className="app-body">
                <main className="app-main">
                    <Suspense
                        fallback={
                            <div className="page-loading">
                                <Spinner size={26} />
                            </div>
                        }
                    >
                        <CurrentPage key={route} />
                    </Suspense>
                </main>
            </div>
            <BottomDock />
            <ToastHost />
            <ModalHost />
        </div>
    );
}
