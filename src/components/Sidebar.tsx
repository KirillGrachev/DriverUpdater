import { useApp, type Route } from '../store/app';
import {
  IconBackups,
  IconDrivers,
  IconHome,
  IconInfo,
  IconSettings,
  IconUpdates,
  type IconProps,
} from './icons';

const NAV: { route: Route; icon: (p: IconProps) => JSX.Element }[] = [
  { route: 'home', icon: IconHome },
  { route: 'drivers', icon: IconDrivers },
  { route: 'updates', icon: IconUpdates },
  { route: 'backups', icon: IconBackups },
  { route: 'settings', icon: IconSettings },
];

export function AppLogo({ size = 40 }: { size?: number }) {
  return (
    <span className="app-logo" style={{ width: size, height: size }}>
      <svg viewBox="0 0 24 24" width={size * 0.62} height={size * 0.62} aria-hidden="true">
        <path
          fill="#fff"
          d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
        />
        <circle cx="12" cy="12" r="3.2" fill="url(#logo-hole)" />
        <defs>
          <linearGradient id="logo-hole" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stopColor="#4d94ff" />
            <stop offset="100%" stopColor="#1d6ef0" />
          </linearGradient>
        </defs>
      </svg>
    </span>
  );
}

export function Sidebar() {
  const { t, route, navigate } = useApp();
  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <AppLogo />
        <span className="sidebar-brand-name">Driver Updater</span>
      </div>
      <nav className="sidebar-nav">
        {NAV.map(({ route: target, icon: Icon }) => (
          <button
            key={target}
            type="button"
            className={`sidebar-item ${route === target ? 'active' : ''}`}
            onClick={() => navigate(target)}
          >
            <Icon size={21} />
            <span>{t(`nav.${target}`)}</span>
          </button>
        ))}
      </nav>
      <div className="sidebar-bottom">
        <button
          type="button"
          className={`sidebar-item ${route === 'about' ? 'active' : ''}`}
          onClick={() => navigate('about')}
        >
          <IconInfo size={21} />
          <span>{t('nav.about')}</span>
        </button>
      </div>
    </aside>
  );
}
