import { useApp, type Route } from '../store/app';
import {
    IconInfoFilled,
    IconMenuBackups,
    IconMenuComponents,
    IconMenuDrivers,
    IconMenuHome,
    IconMenuSettings,
    type IconProps,
} from './icons';

const NAV: { route: Route; icon: (p: IconProps) => JSX.Element }[] = [
    { route: 'home', icon: IconMenuHome },
    { route: 'drivers', icon: IconMenuDrivers },
    { route: 'components', icon: IconMenuComponents },
    { route: 'backups', icon: IconMenuBackups },
    { route: 'settings', icon: IconMenuSettings },
];

export function BottomDock() {
    const { t, route, navigate } = useApp();

    return (
        <div className="bottom-dock">
            <button
                type="button"
                className={`dock-about ${route === 'about' ? 'active' : ''}`}
                title={t('nav.about')}
                aria-label={t('nav.about')}
                onClick={() => navigate('about')}
            >
                <IconInfoFilled size={20} />
            </button>

            <nav className="dock-island">
                {NAV.map(({ route: target, icon: Icon }) => (
                    <button
                        key={target}
                        type="button"
                        className={`dock-item ${route === target ? 'active' : ''}`}
                        aria-current={route === target ? 'page' : undefined}
                        onClick={() => navigate(target)}
                    >
                        <Icon size={18} />
                        <span>{t(`nav.${target}`)}</span>
                    </button>
                ))}
            </nav>
        </div>
    );
}
