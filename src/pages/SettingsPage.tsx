import { useEffect, useState, type ReactNode } from 'react';
import {
  IconBell,
  IconChart,
  IconCheck,
  IconClock,
  IconDatabase,
  IconFolder,
  IconGlobe,
  IconInfo,
  IconMonitor,
  IconMoon,
  IconPalette,
  IconRefresh,
  IconRestore,
  IconShield,
  IconSun,
} from '../components/icons';
import { IllustrationGearHero } from '../components/illustrations';
import { Button, Select, Segmented, Toggle } from '../components/ui';
import { availableLocales } from '../i18n';
import type { CheckInterval, Settings, ThemeMode } from '../ipc/types';
import { useApp } from '../store/app';

function FlagRu() {
  return (
    <svg width="18" height="13" viewBox="0 0 18 13" className="flag" aria-hidden="true">
      <rect width="18" height="13" rx="2" fill="#fff" />
      <rect y="4.33" width="18" height="4.34" fill="#0039a6" />
      <rect y="8.67" width="18" height="4.33" fill="#d52b1e" />
      <rect width="18" height="13" rx="2" fill="none" stroke="rgba(0,0,0,.15)" />
    </svg>
  );
}

function FlagEn() {
  return (
    <svg width="18" height="13" viewBox="0 0 18 13" className="flag" aria-hidden="true">
      <rect width="18" height="13" rx="2" fill="#012169" />
      <path d="M0 0 18 13M18 0 0 13" stroke="#fff" strokeWidth="2.4" />
      <path d="M0 0 18 13M18 0 0 13" stroke="#C8102E" strokeWidth="1" />
      <path d="M9 0v13M0 6.5h18" stroke="#fff" strokeWidth="3.4" />
      <path d="M9 0v13M0 6.5h18" stroke="#C8102E" strokeWidth="1.8" />
      <rect width="18" height="13" rx="2" fill="none" stroke="rgba(0,0,0,.15)" />
    </svg>
  );
}

const FLAGS: Record<string, () => JSX.Element> = { ru: FlagRu, en: FlagEn };

function Row({
  icon,
  label,
  sub,
  children,
}: {
  icon: ReactNode;
  label: string;
  sub?: string;
  children: ReactNode;
}) {
  return (
    <div className="settings-row">
      <div className="settings-row-label">
        <span className="settings-row-icon">{icon}</span>
        <div>
          <div className="settings-row-title">{label}</div>
          {sub ? <div className="settings-row-sub">{sub}</div> : null}
        </div>
      </div>
      <div className="settings-row-control">{children}</div>
    </div>
  );
}

function Section({
  icon,
  title,
  children,
}: {
  icon: ReactNode;
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="card settings-section">
      <div className="settings-section-head">
        <span className="settings-section-icon">{icon}</span>
        <h2>{title}</h2>
      </div>
      {children}
    </section>
  );
}

export function SettingsPage() {
  const { t, data, saveSettings, resetSettings } = useApp();
  const [draft, setDraft] = useState<Settings | null>(data?.settings ?? null);

  useEffect(() => {
    if (data && !draft) setDraft(data.settings);
  }, [data, draft]);

  if (!data || !draft) return null;

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setDraft((prev) => (prev ? { ...prev, [key]: value } : prev));

  const dirty = JSON.stringify(draft) !== JSON.stringify(data.settings);

  return (
    <div className="page page-settings">
      <section className="hero hero-left hero-compact">
        <div className="hero-text">
          <h1 className="hero-title">{t('settings.title')}</h1>
          <p className="hero-sub">{t('settings.subtitle')}</p>
        </div>
        <div className="hero-illu hero-illu-sm">
          <IllustrationGearHero />
        </div>
      </section>

      <Section icon={<IconMonitor size={19} />} title={t('settings.general')}>
        <Row icon={<IconGlobe size={19} />} label={t('settings.language')}>
          <Select
            value={draft.language}
            onChange={(next) => set('language', next)}
            options={availableLocales()
              .filter((l) => l.code === 'ru' || l.code === 'en')
              .map((l) => {
                const Flag = FLAGS[l.code];
                return {
                  value: l.code as Settings['language'],
                  label: l.label,
                  icon: Flag ? <Flag /> : undefined,
                };
              })}
            ariaLabel={t('settings.language')}
          />
        </Row>
        <Row icon={<IconPalette size={19} />} label={t('settings.theme')}>
          <Segmented<ThemeMode>
            value={draft.theme}
            onChange={(next) => set('theme', next)}
            ariaLabel={t('settings.theme')}
            options={[
              { value: 'light', label: t('settings.themeLight'), icon: <IconSun size={15} /> },
              { value: 'dark', label: t('settings.themeDark'), icon: <IconMoon size={15} /> },
              { value: 'system', label: t('settings.themeSystem'), icon: <IconMonitor size={15} /> },
            ]}
          />
        </Row>
        <Row icon={<IconBell size={19} />} label={t('settings.notifications')} sub={t('settings.notifSub')}>
          <Toggle checked={draft.notifications} onChange={(v) => set('notifications', v)} label={t('settings.notifications')} />
        </Row>
      </Section>

      <Section icon={<IconRefresh size={19} />} title={t('settings.secUpdates')}>
        <Row icon={<IconRefresh size={19} />} label={t('settings.autoCheck')} sub={t('settings.autoCheckSub')}>
          <Toggle checked={draft.autoCheck} onChange={(v) => set('autoCheck', v)} label={t('settings.autoCheck')} />
        </Row>
        <Row icon={<IconClock size={19} />} label={t('settings.interval')}>
          <Select<CheckInterval>
            value={draft.checkInterval}
            onChange={(next) => set('checkInterval', next)}
            options={[
              { value: 'hourly', label: t('settings.intervals.hourly') },
              { value: 'daily', label: t('settings.intervals.daily') },
              { value: 'weekly', label: t('settings.intervals.weekly') },
            ]}
            ariaLabel={t('settings.interval')}
          />
        </Row>
      </Section>

      <Section icon={<IconShield size={19} />} title={t('settings.security')}>
        <Row icon={<IconShield size={19} />} label={t('settings.restoreBefore')}>
          <Toggle
            checked={draft.restorePointBeforeInstall}
            onChange={(v) => set('restorePointBeforeInstall', v)}
            label={t('settings.restoreBefore')}
          />
        </Row>
        <Row icon={<IconDatabase size={19} />} label={t('settings.keepBackups')}>
          <Toggle checked={draft.keepBackups} onChange={(v) => set('keepBackups', v)} label={t('settings.keepBackups')} />
        </Row>
        <Row icon={<IconFolder size={19} />} label={t('settings.backupPath')}>
          <div className="path-control">
            <input
              className="text-input"
              value={draft.backupPath}
              onChange={(e) => set('backupPath', e.target.value)}
              aria-label={t('settings.backupPath')}
            />
            <button type="button" className="icon-btn" title={t('settings.browseBtn')} aria-label={t('settings.browseBtn')}>
              <IconFolder size={17} />
            </button>
          </div>
        </Row>
      </Section>

      <Section icon={<IconInfo size={19} />} title={t('settings.extra')}>
        <Row icon={<IconChart size={19} />} label={t('settings.telemetry')} sub={t('settings.telemetrySub')}>
          <Toggle checked={draft.telemetry} onChange={(v) => set('telemetry', v)} label={t('settings.telemetry')} />
        </Row>
      </Section>

      <section className="settings-footer">
        <Button
          variant="ghost"
          icon={<IconRestore size={16} />}
          onClick={() => {
            void resetSettings().then(() => setDraft(null));
          }}
        >
          {t('settings.reset')}
        </Button>
        <Button
          icon={<IconCheck size={17} />}
          disabled={!dirty}
          onClick={() => void saveSettings(draft)}
        >
          {t('settings.save')}
        </Button>
      </section>
    </div>
  );
}
