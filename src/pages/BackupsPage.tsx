import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from 'react';
import { IconCalendar, IconInfoFilled, IconPlusCircle, IconRestoreArrow, IconTrash } from '../components/icons';
import { Button, Select, Spinner, Toggle } from '../components/ui';
import type { DriverBackup, RestorePointPeriod } from '../ipc/types';
import { useApp } from '../store/app';
import { formatDateTime } from '../utils/format';

let backupsCache: DriverBackup[] | null = null;

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
        refreshRestorePoints,
        requestDeleteRestorePoint,
        requestRestorePoint,
        saveSettings,
        listDriverBackups,
        createDriverBackup,
        restoreDriverBackup,
        deleteDriverBackup,
        openModal,
        closeModal,
        pushToast,
    } = useApp();
    const [creating, setCreating] = useState(false);
    const [backups, setBackups] = useState<DriverBackup[]>(() => backupsCache ?? []);
    const [creatingBackup, setCreatingBackup] = useState(false);
    const [busyBackup, setBusyBackup] = useState<string | null>(null);

    useEffect(() => {
        let cancelled = false;
        void Promise.allSettled([
            refreshRestorePoints(),
            listDriverBackups().then((items) => {
                backupsCache = items;
                if (!cancelled) setBackups(items);
            }),
        ]);
        return () => {
            cancelled = true;
        };
    }, [listDriverBackups, refreshRestorePoints]);

    if (!data) return null;

    const settings = data.settings;

    const create = () => {
        setCreating(true);
        void createRestorePoint().finally(() => setCreating(false));
    };

    const pointName = (point: (typeof data.restorePoints)[number]) => point.name;

    const createBackup = async () => {
        setCreatingBackup(true);
        try {
            const backup = await createDriverBackup();
            setBackups((current) => {
                const next = [backup, ...current];
                backupsCache = next;
                return next;
            });
            pushToast(t('backups.backupCreated'), 'success');
        } catch (error) {
            console.error('driver backup failed', error);
            pushToast(t('toasts.error'), 'error');
        } finally {
            setCreatingBackup(false);
        }
    };

    const confirmRestoreBackup = (backup: DriverBackup) => {
        openModal({
            title: t('backups.confirmBackupRestoreTitle'),
            text: t('backups.confirmBackupRestoreText'),
            confirmLabel: t('backups.confirmOk'),
            cancelLabel: t('backups.confirmCancel'),
            tone: 'primary',
            onConfirm: async () => {
                closeModal();
                setBusyBackup(backup.id);
                try {
                    await restoreDriverBackup(backup.id);
                    pushToast(t('backups.backupRestored'), 'success');
                } catch (error) {
                    console.error('driver backup operation failed', error);
                    pushToast(t('toasts.error'), 'error');
                } finally {
                    setBusyBackup(null);
                }
            },
        });
    };

    const confirmDeleteBackup = (backup: DriverBackup) => {
        openModal({
            title: t('backups.confirmBackupDeleteTitle'),
            text: t('backups.confirmBackupDeleteText', { name: backup.id }),
            confirmLabel: t('backups.confirmOk'),
            cancelLabel: t('backups.confirmCancel'),
            tone: 'danger',
            onConfirm: async () => {
                closeModal();
                setBusyBackup(backup.id);
                try {
                    await deleteDriverBackup(backup.id);
                    setBackups((current) => {
                        const next = current.filter((item) => item.id !== backup.id);
                        backupsCache = next;
                        return next;
                    });
                    pushToast(t('backups.backupDeleted'), 'info');
                } catch (error) {
                    console.error('driver backup operation failed', error);
                    pushToast(t('toasts.error'), 'error');
                } finally {
                    setBusyBackup(null);
                }
            },
        });
    };

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
                                            {point.sizeMb > 0
                                                ? `~ ${point.sizeMb} ${t('common.mb')}`
                                                : t('common.never')}
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

            <section className="card backups-card">
                <div className="card-head">
                    <div>
                        <h2 className="card-title">{t('backups.driverBackups')}</h2>
                        <p className="card-sub">{t('backups.driverBackupsSub')}</p>
                    </div>
                    <Button
                        variant="secondary"
                        size="sm"
                        icon={<IconPlusCircle size={16} />}
                        loading={creatingBackup}
                        onClick={() => void createBackup()}
                    >
                        {creatingBackup ? t('backups.creatingBackup') : t('backups.createBackup')}
                    </Button>
                </div>
                {backups.length === 0 ? (
                    <div className="backups-empty">
                        <div className="backups-empty-title">{t('backups.backupEmpty')}</div>
                        <div className="backups-empty-sub">{t('backups.backupEmptySub')}</div>
                    </div>
                ) : (
                    <ul className="backup-list">
                        {backups.map((backup) => {
                            const busy = busyBackup === backup.id;
                            return (
                                <li key={backup.id} className="backup-row">
                                    <div className="backup-ident">
                                        <div className="backup-name">{backup.id}</div>
                                        <div className="backup-kind">{backup.path}</div>
                                    </div>
                                    <div className="backup-date">{formatDateTime(backup.createdAt, locale)}</div>
                                    <div className="backup-size">
                                        {backup.sizeMb > 0 ? `~ ${backup.sizeMb} ${t('common.mb')}` : t('common.never')}
                                        <br />
                                        <small>{t('backups.backupPackages', { n: backup.packageCount })}</small>
                                    </div>
                                    <Button
                                        variant="secondary"
                                        size="sm"
                                        icon={<IconRestoreArrow size={15} />}
                                        loading={busy}
                                        onClick={() => confirmRestoreBackup(backup)}
                                    >
                                        {t('backups.restoreBackup')}
                                    </Button>
                                    <button
                                        type="button"
                                        className="backup-delete"
                                        title={t('backups.deleteBackup')}
                                        aria-label={t('backups.deleteBackup')}
                                        disabled={busy}
                                        onClick={() => confirmDeleteBackup(backup)}
                                    >
                                        <IconTrash size={17} />
                                    </button>
                                </li>
                            );
                        })}
                    </ul>
                )}
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
                            className="period-select"
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
