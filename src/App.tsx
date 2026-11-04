import { Sidebar } from './components/Sidebar';
import { TitleBar } from './components/TitleBar';
import { ModalHost, ToastHost } from './components/overlays';
import { Spinner } from './components/ui';
import { useApp } from './store/app';

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
          <div className="page">
            <section className="card empty-card">
              <h2>Каркас интерфейса готов</h2>
              <p>Страницы подключаются следующими коммитами.</p>
            </section>
          </div>
        </main>
      </div>
      <ToastHost />
      <ModalHost />
    </div>
  );
}
