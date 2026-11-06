import { useEffect, useRef, useState } from 'react';
import logo from '../assets/branding/logo.png';

/**
 * The single loading surface used during boot and page work.
 *
 * @param visible whether work is still in flight
 * @param label optional localized status line
 *
 * <p>The element stays mounted while it fades, so boot, scan and page loads
 * share one visual instead of swapping between different spinners.</p>
 */
/**
 * Keeps short loading work invisible and prevents a shown loader from flashing.
 *
 * @param active whether work is in flight
 * @param delay time to wait before showing the loader
 * @param minVisible minimum time the loader remains visible once shown
 * @returns whether the loader should currently be rendered
 */
export function useDelayedLoading(active: boolean, delay = 120, minVisible = 500) {
    const [shown, setShown] = useState(false);
    const shownAt = useRef(0);
    const timer = useRef<number | undefined>(undefined);

    useEffect(() => {
        window.clearTimeout(timer.current);
        if (active) {
            if (shown) return undefined;
            timer.current = window.setTimeout(() => {
                shownAt.current = performance.now();
                setShown(true);
            }, delay);
        } else if (shown) {
            const elapsed = performance.now() - shownAt.current;
            timer.current = window.setTimeout(() => setShown(false), Math.max(0, minVisible - elapsed));
        }
        return () => window.clearTimeout(timer.current);
    }, [active, delay, minVisible, shown]);

    return shown;
}

export function LoadingScreen({ visible, label }: { visible: boolean; label?: string }) {
    return (
        <div
            className={`loading-screen ${visible ? 'is-visible' : 'is-hidden'}`}
            role="status"
            aria-live="polite"
            aria-label={label ?? 'Loading'}
            aria-hidden={!visible}
        >
            <div className="loading-screen-inner">
                <div className="loading-screen-mark">
                    <img src={logo} alt="" aria-hidden="true" draggable={false} />
                </div>
                <div className="loading-screen-copy">
                    <strong>Driver Updater</strong>
                </div>
                <div className="loading-screen-bar" aria-hidden="true">
                    <i />
                </div>
            </div>
        </div>
    );
}
