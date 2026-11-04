import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from 'react';
import { IconCheck, IconChevronDown } from './icons';

/** Animated height container for disclosure panels. */

export function Collapse({ open, children }: { open: boolean; children: ReactNode }) {
    const innerRef = useRef<HTMLDivElement>(null);
    const [height, setHeight] = useState(0);

    useLayoutEffect(() => {
        if (innerRef.current) {
            setHeight(open ? innerRef.current.scrollHeight : 0);
        }
    }, [open, children]);

    useEffect(() => {
        const inner = innerRef.current;
        if (!inner || !open) return undefined;
        const observer = new ResizeObserver(() => setHeight(inner.scrollHeight));
        observer.observe(inner);
        return () => observer.disconnect();
    }, [open]);

    return (
        <div className="collapse" style={{ height }} aria-hidden={!open}>
            <div ref={innerRef}>{children}</div>
        </div>
    );
}

/** Primary button component. */

export interface ButtonProps {
    variant?: 'primary' | 'secondary' | 'ghost' | 'outline' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    icon?: ReactNode;
    children?: ReactNode;
    onClick?: () => void;
    disabled?: boolean;
    loading?: boolean;
    title?: string;
    href?: string;
    className?: string;
}

export function Spinner({ size = 16 }: { size?: number }) {
    return (
        <svg className="spinner" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="12" cy="12" r="9" stroke="currentColor" strokeWidth="3" opacity="0.25" fill="none" />
            <path d="M21 12a9 9 0 0 0-9-9" stroke="currentColor" strokeWidth="3" strokeLinecap="round" fill="none" />
        </svg>
    );
}

export function Button({
    variant = 'primary',
    size = 'md',
    icon,
    children,
    onClick,
    disabled,
    loading,
    title,
    href,
    className = '',
}: ButtonProps) {
    const classes = `btn btn-${variant} btn-${size} ${className}`.trim();
    const content = (
        <>
            {loading ? <Spinner size={size === 'sm' ? 14 : 16} /> : icon}
            {children ? <span className="btn-label">{children}</span> : null}
        </>
    );
    if (href) {
        return (
            <a className={classes} href={href} target="_blank" rel="noreferrer noopener" title={title}>
                {content}
            </a>
        );
    }
    return (
        <button type="button" className={classes} onClick={onClick} disabled={disabled || loading} title={title}>
            {content}
        </button>
    );
}

/** Accessible switch. */

export function Toggle({
    checked,
    onChange,
    label,
    disabled,
}: {
    checked: boolean;
    onChange: (next: boolean) => void;
    label?: string;
    disabled?: boolean;
}) {
    return (
        <button
            type="button"
            role="switch"
            aria-checked={checked}
            aria-label={label}
            className={`toggle ${checked ? 'on' : ''}`}
            disabled={disabled}
            onClick={() => onChange(!checked)}
        >
            <span className="toggle-thumb" />
        </button>
    );
}

/** Checkbox control. */

export function Checkbox({
    checked,
    indeterminate,
    onChange,
    label,
    disabled,
}: {
    checked: boolean;
    indeterminate?: boolean;
    onChange: (next: boolean) => void;
    label?: string;
    disabled?: boolean;
}) {
    return (
        <button
            type="button"
            role="checkbox"
            aria-checked={indeterminate ? 'mixed' : checked}
            aria-label={label}
            className={`checkbox ${checked || indeterminate ? 'on' : ''}`}
            disabled={disabled}
            onClick={() => onChange(!checked)}
        >
            {indeterminate ? (
                <span className="checkbox-dash" />
            ) : checked ? (
                <IconCheck size={13} strokeWidth={3} />
            ) : null}
        </button>
    );
}

/** Dropdown select with keyboard dismissal. */

export interface SelectOption<T extends string> {
    value: T;
    label: string;
    icon?: ReactNode;
}

export function Select<T extends string>({
    value,
    onChange,
    options,
    icon,
    className = '',
    ariaLabel,
    onOpen,
}: {
    value: T;
    onChange: (next: T) => void;
    options: SelectOption<T>[];
    icon?: ReactNode;
    className?: string;
    ariaLabel?: string;
    onOpen?: () => void;
}) {
    const [open, setOpen] = useState(false);
    const rootRef = useRef<HTMLDivElement>(null);
    const current = options.find((o) => o.value === value);

    const toggle = () => {
        if (!open) {
            onOpen?.();
            const root = rootRef.current;
            const scroller = root?.closest('.app-main') as HTMLElement | null;
            const rect = root?.getBoundingClientRect();
            const menuHeight = Math.min(400, options.length * 36 + 8);
            if (scroller && rect) {
                const room = scroller.getBoundingClientRect().bottom - rect.bottom;
                if (room < menuHeight + 10) scroller.scrollTop += menuHeight + 10 - room;
            }
        }
        setOpen((prev) => !prev);
    };

    useEffect(() => {
        if (!open) return undefined;
        const onPointerDown = (event: MouseEvent) => {
            if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
        };
        const onKeyDown = (event: KeyboardEvent) => {
            if (event.key === 'Escape') setOpen(false);
        };
        document.addEventListener('mousedown', onPointerDown);
        document.addEventListener('keydown', onKeyDown);
        return () => {
            document.removeEventListener('mousedown', onPointerDown);
            document.removeEventListener('keydown', onKeyDown);
        };
    }, [open]);

    return (
        <div ref={rootRef} className={`select ${open ? 'open' : ''} ${className}`.trim()}>
            <button
                type="button"
                className="select-trigger"
                aria-haspopup="listbox"
                aria-expanded={open}
                aria-label={ariaLabel}
                onClick={toggle}
            >
                {icon ? <span className="select-icon">{icon}</span> : null}
                {current?.icon ? <span className="select-icon">{current.icon}</span> : null}
                <span className="select-value">{current?.label}</span>
                <IconChevronDown size={16} className="select-chevron" />
            </button>
            {open ? (
                <ul className="select-menu" role="listbox" aria-label={ariaLabel}>
                    {options.map((option) => (
                        <li key={option.value}>
                            <button
                                type="button"
                                role="option"
                                aria-selected={option.value === value}
                                className={`select-option ${option.value === value ? 'active' : ''}`}
                                onClick={() => {
                                    onChange(option.value);
                                    setOpen(false);
                                }}
                            >
                                {option.icon ? <span className="select-icon">{option.icon}</span> : null}
                                <span>{option.label}</span>
                                {option.value === value ? <IconCheck size={14} className="select-check" /> : null}
                            </button>
                        </li>
                    ))}
                </ul>
            ) : null}
        </div>
    );
}

/** Segmented control. */

export function Segmented<T extends string>({
    value,
    onChange,
    options,
    ariaLabel,
}: {
    value: T;
    onChange: (next: T) => void;
    options: SelectOption<T>[];
    ariaLabel?: string;
}) {
    return (
        <div className="segmented" role="radiogroup" aria-label={ariaLabel}>
            {options.map((o) => (
                <button
                    key={o.value}
                    type="button"
                    role="radio"
                    aria-checked={o.value === value}
                    className={`segmented-item ${o.value === value ? 'active' : ''}`}
                    onClick={() => onChange(o.value)}
                >
                    {o.icon}
                    <span>{o.label}</span>
                </button>
            ))}
        </div>
    );
}

/** Status badge. */

export function Badge({
    tone = 'neutral',
    children,
}: {
    tone?: 'info' | 'neutral' | 'success' | 'warn';
    children: ReactNode;
}) {
    return <span className={`badge badge-${tone}`}>{children}</span>;
}

/** Summary statistic card. */

export function StatCard({
    icon,
    tone = 'blue',
    value,
    label,
    action,
}: {
    icon: ReactNode;
    tone?: 'blue' | 'green' | 'orange' | 'slate';
    value: ReactNode;
    label: ReactNode;
    action?: ReactNode;
}) {
    return (
        <div className="stat-card">
            <span className={`stat-icon tone-${tone}`}>{icon}</span>
            <div className="stat-text">
                <div className="stat-value">{value}</div>
                <div className="stat-label">{label}</div>
            </div>
            {action ? <div className="stat-action">{action}</div> : null}
        </div>
    );
}
