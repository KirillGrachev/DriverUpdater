import logo from '../assets/branding/logo.png';
import { Spinner } from './ui';

/**
 * One loading screen for the whole app.
 *
 * @param visible whether work is still in flight
 *
 * <p>The element stays mounted while it fades, so route and page loads never
 * flash a different spinner or a skeleton inside every card.</p>
 */
export function LoadingScreen({ visible }: { visible: boolean }) {
    return (
        <div
            className={`loading-screen ${visible ? 'is-visible' : 'is-hidden'}`}
            role="status"
            aria-live="polite"
            aria-label="Loading"
            aria-hidden={!visible}
        >
            <div className="loading-screen-inner">
                <img src={logo} alt="" aria-hidden="true" draggable={false} />
                <Spinner size={28} />
            </div>
        </div>
    );
}
