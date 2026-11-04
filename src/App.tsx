import { useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { BottomDock } from './components/BottomDock';
import { TitleBar } from './components/TitleBar';
import { ModalHost, ToastHost } from './components/overlays';
import { Button, Spinner } from './components/ui';
import { AboutPage } from './pages/AboutPage';
import { BackupsPage } from './pages/BackupsPage';
import { ComponentsPage } from './pages/ComponentsPage';
import { DriversPage } from './pages/DriversPage';
import { HomePage } from './pages/HomePage';
import { SettingsPage } from './pages/SettingsPage';
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
        if (booting) return;
        /**
         * The native window starts hidden so the webview never paints a white
         * frame while the theme and initial data are still loading.
         */
        if ('__TAURI_INTERNALS__' in window) {
            void getCurrentWindow().show();
        }
    }, [booting]);

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
                    <CurrentPage key={route} />
                </main>
            </div>
            <BottomDock />
            <ToastHost />
            <ModalHost />
        </div>
    );
}
