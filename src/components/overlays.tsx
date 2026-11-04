import { useEffect } from 'react';
import { useApp } from '../store/app';
import { IconAlert, IconCheckCircleFilled, IconClose, IconInfoFilled } from './icons';
import { Button } from './ui';

export function ToastHost() {
  const { toasts, dismissToast } = useApp();
  if (toasts.length === 0) return null;
  return (
    <div className="toast-host" role="status" aria-live="polite">
      {toasts.map((toast) => (
        <div key={toast.id} className={`toast toast-${toast.tone} ${toast.leaving ? 'leaving' : ''}`}>
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
          <button type="button" className="toast-close" onClick={() => dismissToast(toast.id)}>
            <IconClose size={14} />
          </button>
        </div>
      ))}
    </div>
  );
}

export function ModalHost() {
  const { modal, closeModal } = useApp();

  useEffect(() => {
    if (!modal) return undefined;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') closeModal();
    };
    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, [modal, closeModal]);

  if (!modal) return null;
  return (
    <div className="modal-backdrop" onClick={closeModal}>
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-label={modal.title}
        onClick={(e) => e.stopPropagation()}
      >
        <h3 className="modal-title">{modal.title}</h3>
        <p className="modal-text">{modal.text}</p>
        <div className="modal-actions">
          <Button variant="ghost" onClick={closeModal}>
            {modal.cancelLabel}
          </Button>
          <Button variant={modal.tone === 'danger' ? 'danger' : 'primary'} onClick={() => void modal.onConfirm()}>
            {modal.confirmLabel}
          </Button>
        </div>
      </div>
    </div>
  );
}
