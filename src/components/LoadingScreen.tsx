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
                    <span>{label ?? 'Loading...'}</span>
                </div>
                <div className="loading-screen-bar" aria-hidden="true">
                    <i />
                </div>
            </div>
        </div>
    );
}
