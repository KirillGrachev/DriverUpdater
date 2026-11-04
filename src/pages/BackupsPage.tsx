import { useState } from 'react';
import {
  IconCalendar,
  IconInfoFilled,
  IconPlusCircle,
  IconRestore,
  IconTrash,
} from '../components/icons';
import { IllustrationDriveClock } from '../components/illustrations';
import { Button, Select, Toggle } from '../components/ui';
import type { RestorePointPeriod } from '../ipc/types';
import { useApp } from '../store/app';
import { formatDateTime } from '../utils/format';

export function BackupsPage() {
  const { t, locale, data, busyRestorePoints, createRestorePoint, requestDeleteRestorePoint, requestRestorePoint, saveSettings } =
    useApp();
  const [creating, setCreating] = useState(false);

  if (!data) return null;

  const settings = data.settings;

  const create = () => {
    setCreating(true);
    void createRestorePoint().finally(() => setCreating(false));
  };

  const pointName = (point: (typeof data.restorePoints)[number]) =>
    point.name[locale === 'ru' ? 'ru' : 'en'];

  return (
    <div className="page page-backups">
      <section className="hero hero-left">
        <div className="hero-text">
          <h1 className="hero-title">{t('backups.title')}</h1>
          <p className="hero-sub">{t('backups.subtitle')}</p>
        </div>
        <div className="hero-illu">
          <IllustrationDriveClock />
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
        <ul className="backup-list">
          {data.restorePoints.map((point) => {
            const busy = busyRestorePoints.includes(point.id);
            return (
              <li key={point.id} className="backup-row">
                <span className="backup-icon">
                  <IconRestore size={24} />
                </span>
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
                  icon={<IconRestore size={15} />}
                  loading={busy}
                  onClick={() => requestRestorePoint(point.id, pointName(point))}
                >
                  {busy ? t('backups.restoringBtn') : t('backups.restoreBtn')}
                </Button>
                <button
                  type="button"
                  className="icon-btn icon-btn-danger"
                  title={t('backups.deleteBtn')}
                  aria-label={t('backups.deleteBtn')}
                  disabled={busy}
                  onClick={() => requestDeleteRestorePoint(point.id, pointName(point))}
                >
                  <IconTrash size={17} />
                </button>
              </li>
            );
          })}
        </ul>
      </section>

      <section className="card auto-card">
        <h2 className="card-title">{t('backups.autoTitle')}</h2>
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
              icon={<IconCalendar size={16} />}
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
