import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from 'react';
import { IconCalendar, IconInfoFilled, IconPlusCircle, IconRestoreArrow, IconTrash } from '../components/icons';
import { Button, Select, Spinner, Toggle } from '../components/ui';
import type { RestorePointPeriod } from '../ipc/types';
import { useApp } from '../store/app';
import { formatDateTime } from '../utils/format';

function AnimatedHeight({ children }: { children: ReactNode }) {
    const innerRef = useRef<HTMLDivElement>(null);
    const [height, setHeight] = useState<number | undefined>(undefined);

    useLayoutEffect(() => {
        if (innerRef.current) setHeight(innerRef.current.scrollHeight);
    }, [children]);

    useEffect(() => {
        const inner = innerRef.current;
        if (!inner) return undefined;
        const observer = new ResizeObserver(() => setHeight(inner.scrollHeight));
        observer.observe(inner);
        return () => observer.disconnect();
    }, []);

    return (
        <div className="animated-height" style={height === undefined ? undefined : { height }}>
            <div ref={innerRef}>{children}</div>
        </div>
    );
}

export function BackupsPage() {
    const {
        t,
        locale,
        data,
        busyRestorePoints,
        busyDeletePoints,
        createRestorePoint,
        requestDeleteRestorePoint,
        requestRestorePoint,
        saveSettings,
    } = useApp();
    const [creating, setCreating] = useState(false);

    if (!data) return null;

    const settings = data.settings;

    const create = () => {
        setCreating(true);
        void createRestorePoint().finally(() => setCreating(false));
    };

    const pointName = (point: (typeof data.restorePoints)[number]) => point.name[locale === 'ru' ? 'ru' : 'en'];

    return (
        <div className="page page-backups">
            <section className="hero hero-left">
                <div className="hero-text">
                    <h1 className="hero-title">{t('backups.title')}</h1>
                    <p className="hero-sub">{t('backups.subtitle')}</p>
                </div>
            </section>

            <section className="notice-bar notice-banner">
                <IconInfoFilled size={30} className="notice-icon-blue" />
                <div>
                    <div className="notice-title">{t('backups.banner')}</div>
                    <div className="notice-sub">{t('backups.bannerSub')}</div>
                </div>
            </section>

            <section className="card backups-card">
                <div className="card-head">
                    <h2 className="card-title">{t('backups.pointsTitle')}</h2>
                    <Button
                        variant="secondary"
                        size="sm"
                        icon={<IconPlusCircle size={16} />}
                        loading={creating}
                        onClick={create}
                    >
                        {creating ? t('backups.creatingBtn') : t('backups.createBtn')}
                    </Button>
                </div>
                <AnimatedHeight>
                    {data.restorePoints.length === 0 ? (
                        <div className="backups-empty">
                            <div className="backups-empty-title">{t('backups.emptyTitle')}</div>
                            <div className="backups-empty-sub">{t('backups.emptySub')}</div>
                        </div>
                    ) : (
                        <ul className="backup-list">
                            {data.restorePoints.map((point) => {
                                const restoring = busyRestorePoints.includes(point.id);
                                const deleting = busyDeletePoints.includes(point.id);
                                return (
                                    <li key={point.id} className="backup-row">
                                        <div className="backup-ident">
                                            <div className="backup-name">{pointName(point)}</div>
                                            <div className="backup-kind">
                                                {point.kind === 'auto'
                                                    ? t('backups.kindAuto')
                                                    : point.kind === 'schedule'
                                                      ? t('backups.kindSchedule')
                                                      : t('backups.kindManual')}
                                            </div>
                                        </div>
                                        <div className="backup-date">{formatDateTime(point.createdAt, locale)}</div>
                                        <div className="backup-size">
                                            ~ {point.sizeMb} {t('common.mb')}
                                        </div>
                                        <Button
                                            variant="secondary"
                                            size="sm"
                                            icon={<IconRestoreArrow size={15} />}
                                            loading={restoring}
                                            disabled={deleting}
                                            onClick={() => requestRestorePoint(point.id, pointName(point))}
                                        >
                                            {restoring ? t('backups.restoringBtn') : t('backups.restoreBtn')}
                                        </Button>
                                        <button
                                            type="button"
                                            className="backup-delete"
                                            title={t('backups.deleteBtn')}
                                            aria-label={t('backups.deleteBtn')}
                                            disabled={restoring || deleting}
                                            onClick={() => requestDeleteRestorePoint(point.id, pointName(point))}
                                        >
                                            {deleting ? <Spinner size={16} /> : <IconTrash size={17} />}
                                        </button>
                                    </li>
                                );
                            })}
                        </ul>
                    )}
                </AnimatedHeight>
            </section>

            <section className="card auto-card">
                <div className="card-head">
                    <h2 className="card-title">{t('backups.autoTitle')}</h2>
                </div>
                <div className="auto-row">
                    <div className="auto-left">
                        <Toggle
                            checked={settings.autoRestorePoints}
                            label={t('backups.autoToggle')}
                            onChange={(next) => void saveSettings({ ...settings, autoRestorePoints: next })}
                        />
                        <div>
                            <div className="auto-label">{t('backups.autoToggle')}</div>
                            <div className="auto-sub">{t('backups.autoSub')}</div>
                        </div>
                    </div>
                    <div className="auto-right">
                        <div className="auto-period-label">{t('backups.periodLabel')}</div>
                        <Select<RestorePointPeriod>
                            icon={<IconCalendar size={18} />}
                            value={settings.restorePointPeriod}
                            onChange={(next) => void saveSettings({ ...settings, restorePointPeriod: next })}
                            options={[
                                { value: 'daily', label: t('backups.periods.daily') },
                                { value: 'weekly', label: t('backups.periods.weekly') },
                                { value: 'monthly', label: t('backups.periods.monthly') },
                            ]}
                            ariaLabel={t('backups.periodLabel')}
                        />
                    </div>
                </div>
            </section>
        </div>
    );
}
