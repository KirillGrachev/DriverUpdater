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
    }

    render(): ReactNode {
        if (!this.state.error) return this.props.children;
        return (
            <section className="card boot-failure-card page-render-error" role="alert">
                <h1>Something went wrong</h1>
                <p>{this.state.error.message}</p>
                <button type="button" className="btn btn-primary btn-md" onClick={() => window.location.reload()}>
                    Reload
                </button>
            </section>
        );
    }
}
