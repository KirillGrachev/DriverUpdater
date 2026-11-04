import { Sidebar } from './components/Sidebar';
import { TitleBar } from './components/TitleBar';
import { ModalHost, ToastHost } from './components/overlays';
import { Spinner } from './components/ui';
import { AboutPage } from './pages/AboutPage';
import { BackupsPage } from './pages/BackupsPage';
import { DriversPage } from './pages/DriversPage';
import { HomePage } from './pages/HomePage';
import { SettingsPage } from './pages/SettingsPage';
import { UpdatesPage } from './pages/UpdatesPage';
import { useApp } from './store/app';

function CurrentPage() {
  const { route } = useApp();
  switch (route) {
    case 'home':
      return <HomePage />;
    case 'drivers':
      return <DriversPage />;
    case 'updates':
      return <UpdatesPage />;
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
  const { booting } = useApp();

  if (booting) {
    return (
      <div className="app-window boot-window">
        <div className="boot-splash">
          <Spinner size={34} />
        </div>
      </div>
    );
  }

  return (
    <div className="app-window">
      <TitleBar />
      <div className="app-body">
        <Sidebar />
        <main className="app-main">
          <CurrentPage />
        </main>
      </div>
      <ToastHost />
      <ModalHost />
    </div>
  );
}
