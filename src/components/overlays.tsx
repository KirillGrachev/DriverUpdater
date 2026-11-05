/* eslint-disable jsx-a11y/click-events-have-key-events, jsx-a11y/no-static-element-interactions, jsx-a11y/no-noninteractive-element-interactions */
import { useEffect, useRef, useState } from 'react';
import { useApp } from '../store/app';
import { IconAlert, IconCheckCircleFilled, IconClose, IconInfoFilled } from './icons';
import { Button } from './ui';

export function ToastHost() {
    const { toasts, dismissToast, t } = useApp();
    if (toasts.length === 0) return null;
    return (
        <div className="toast-host" role="status" aria-live="polite">
            {toasts.map((toast) => (
                <div
                    key={toast.id}
                    className={`toast toast-${toast.tone} ${toast.leaving ? 'leaving' : ''}`}
                    role={toast.tone === 'error' ? 'alert' : undefined}
                >
                    <span className="toast-icon">
                        {toast.tone === 'success' ? (
                            <IconCheckCircleFilled size={20} />
                        ) : toast.tone === 'error' ? (
                            <IconAlert size={20} />
                        ) : (
                            <IconInfoFilled size={20} />
                        )}
                    </span>
                    <span className="toast-text">{toast.text}</span>
                    <button
                        type="button"
                        className="toast-close"
                        aria-label={t('common.dismiss')}
                        onClick={() => dismissToast(toast.id)}
                    >
                        <IconClose size={14} />
                    </button>
                </div>
            ))}
        </div>
    );
}

export function ModalHost() {
    const { modal, closeModal } = useApp();
    const modalRef = useRef<HTMLDivElement>(null);
    const [busy, setBusy] = useState(false);
    const busyRef = useRef(false);

    useEffect(() => {
        if (!modal) return undefined;
        const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
        const controls = modalRef.current?.querySelectorAll<HTMLElement>('button');
        controls?.[0]?.focus();
        const onKeyDown = (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                event.preventDefault();
                if (!busyRef.current) closeModal();
                return;
            }
            if (event.key !== 'Tab' || !modalRef.current) return;
            const focusable = Array.from(modalRef.current.querySelectorAll<HTMLElement>('button:not(:disabled)'));
            if (focusable.length === 0) return;
            const first = focusable[0];
            const last = focusable[focusable.length - 1];
            if (event.shiftKey && document.activeElement === first) {
                event.preventDefault();
                last.focus();
            } else if (!event.shiftKey && document.activeElement === last) {
                event.preventDefault();
                first.focus();
            }
        };
        document.addEventListener('keydown', onKeyDown);
        return () => {
            document.removeEventListener('keydown', onKeyDown);
            previous?.focus();
        };
    }, [modal, closeModal]);

    const confirm = async () => {
        if (!modal || busy) return;
        busyRef.current = true;
        setBusy(true);
        try {
            await modal.onConfirm();
        } finally {
            busyRef.current = false;
            setBusy(false);
        }
    };

    if (!modal) return null;
    return (
        <div
            className="modal-backdrop"
            onClick={() => {
                if (!busyRef.current) closeModal();
            }}
        >
            <div
                ref={modalRef}
                className="modal"
                role="dialog"
                aria-modal="true"
                aria-label={modal.title}
                onClick={(e) => e.stopPropagation()}
            >
                <h3 className="modal-title">{modal.title}</h3>
                <p className="modal-text">{modal.text}</p>
                <div className="modal-actions">
                    <Button variant="ghost" disabled={busy} onClick={closeModal}>
                        {modal.cancelLabel}
                    </Button>
                    <Button
                        variant={modal.tone === 'danger' ? 'danger' : 'primary'}
                        loading={busy}
                        onClick={() => void confirm()}
                    >
                        {modal.confirmLabel}
                    </Button>
                </div>
            </div>
        </div>
    );
}
