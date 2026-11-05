import { Component, type ErrorInfo, type ReactNode } from 'react';

interface Props {
    children: ReactNode;
}

interface State {
    error: Error | null;
}

/** Keeps a rendering failure from blanking the whole desktop shell. */
export class ErrorBoundary extends Component<Props, State> {
    state: State = { error: null };

    static getDerivedStateFromError(error: Error): State {
        return { error };
    }

    componentDidCatch(error: Error, info: ErrorInfo): void {
        console.error('page render failed', error, info.componentStack);
        const isChunkFailure = /dynamically imported module|Importing a module script failed|Failed to fetch/i.test(
            error.message,
        );
        if (isChunkFailure && !sessionStorage.getItem('driver-updater-chunk-reload')) {
            // A stale lazy chunk usually means the app was updated while the
            // webview was open. Reload once instead of showing a developer error.
            sessionStorage.setItem('driver-updater-chunk-reload', '1');
            window.location.reload();
        }
    }

    render(): ReactNode {
        if (!this.state.error) return this.props.children;
        return (
            <section className="card boot-failure-card page-render-error" role="alert">
                <h1>Something went wrong</h1>
                <p>The page could not be loaded. Reload the app to continue.</p>
                <button type="button" className="btn btn-primary btn-md" onClick={() => window.location.reload()}>
                    Reload
                </button>
            </section>
        );
    }
}
