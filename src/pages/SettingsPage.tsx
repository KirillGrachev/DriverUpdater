import type { ReactNode } from 'react';
import {
    IconLineBell,
    IconLineCards,
    IconLineChart,
    IconLineClock,
    IconLineDatabase,
    IconLineFolder,
    IconLineGlobe,
    IconLinePalette,
    IconLineShield,
    IconLineSync,
} from '../components/icons';
import { Select, Segmented, Toggle } from '../components/ui';
import { LocaleFlag } from '../components/Flags';
import sunIcon from '../assets/icons/icon-sun.svg';
import moonIcon from '../assets/icons/icon-moon.svg';
import systemIcon from '../assets/icons/icon-sun-moon.svg';
import { availableLocales } from '../i18n';
import type { CheckInterval, Settings, ThemeMode } from '../ipc/types';
import { useApp } from '../store/app';

function Row({ icon, label, sub, children }: { icon: ReactNode; label: string; sub?: string; children: ReactNode }) {
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

function Section({ title, children }: { title: string; children: ReactNode }) {
    return (
        <section className="card settings-section">
            <div className="settings-section-head">
                <h2>{title}</h2>
            </div>
            {children}
        </section>
    );
}

export function SettingsPage() {
    const { t, data, updateSettings, applyTheme, applyLanguage, openBackupFolder, pushToast } = useApp();

    if (!data) return null;

    const settings = data.settings;
    const update = <K extends keyof Settings>(key: K, value: Settings[K]) => {
        void updateSettings({ [key]: value });
    };

    return (
        <div className="page page-settings">
            <section className="hero hero-left hero-compact">
                <div className="hero-text">
                    <h1 className="hero-title">{t('settings.title')}</h1>
                    <p className="hero-sub">{t('settings.subtitle')}</p>
                </div>
            </section>

            <Section title={t('settings.general')}>
                <Row icon={<IconLineGlobe className="settings-line-icon" />} label={t('settings.language')}>
                    <Select
                        className="language-select"
                        value={settings.language}
                        onChange={(next) => {
                            void applyLanguage(next);
                        }}
                        options={availableLocales().map((locale) => ({
                            value: locale.code,
                            label: locale.label,
                            icon: <LocaleFlag code={locale.code} />,
                        }))}
                        ariaLabel={t('settings.language')}
                    />
                </Row>
                <Row icon={<IconLinePalette className="settings-line-icon" />} label={t('settings.theme')}>
                    <Segmented<ThemeMode>
                        value={settings.theme}
                        onChange={(next) => {
                            void applyTheme(next);
                        }}
                        ariaLabel={t('settings.theme')}
                        options={[
                            {
                                value: 'light',
                                label: t('settings.themeLight'),
                                icon: <img className="theme-icon" src={sunIcon} alt="" />,
                            },
                            {
                                value: 'dark',
                                label: t('settings.themeDark'),
                                icon: <img className="theme-icon" src={moonIcon} alt="" />,
                            },
                            {
                                value: 'system',
                                label: t('settings.themeSystem'),
                                icon: <img className="theme-icon" src={systemIcon} alt="" />,
                            },
                        ]}
                    />
                </Row>
                <Row
                    icon={<IconLineBell className="settings-line-icon" />}
                    label={t('settings.notifications')}
                    sub={t('settings.notifSub')}
                >
                    <Toggle
                        checked={settings.notifications}
                        onChange={(v) => update('notifications', v)}
                        label={t('settings.notifications')}
                    />
                </Row>
            </Section>

            <Section title={t('settings.secUpdates')}>
                <Row
                    icon={<IconLineSync className="settings-line-icon" />}
                    label={t('settings.autoCheck')}
                    sub={t('settings.autoCheckSub')}
                >
                    <Toggle
                        checked={settings.autoCheck}
                        onChange={(v) => update('autoCheck', v)}
                        label={t('settings.autoCheck')}
                    />
                </Row>
                <Row icon={<IconLineClock className="settings-line-icon" />} label={t('settings.interval')}>
                    <Select<CheckInterval>
                        value={settings.checkInterval}
                        onChange={(next) => update('checkInterval', next)}
                        options={[
                            { value: 'hourly', label: t('settings.intervals.hourly') },
                            { value: 'daily', label: t('settings.intervals.daily') },
                            { value: 'weekly', label: t('settings.intervals.weekly') },
                        ]}
                        ariaLabel={t('settings.interval')}
                    />
                </Row>
            </Section>

            <Section title={t('settings.security')}>
                <Row icon={<IconLineShield className="settings-line-icon" />} label={t('settings.restoreBefore')}>
                    <Toggle
                        checked={settings.restorePointBeforeInstall}
                        onChange={(v) => update('restorePointBeforeInstall', v)}
                        label={t('settings.restoreBefore')}
                    />
                </Row>
                <Row icon={<IconLineDatabase className="settings-line-icon" />} label={t('settings.keepBackups')}>
                    <Toggle
                        checked={settings.keepBackups}
                        onChange={(v) => update('keepBackups', v)}
                        label={t('settings.keepBackups')}
                    />
                </Row>
                <Row icon={<IconLineFolder className="settings-line-icon" />} label={t('settings.backupPath')}>
                    <div className="path-control">
                        <div className="path-readonly path-value" title={settings.backupPath}>
                            {settings.backupPath}
                        </div>
                        <button
                            type="button"
                            className="icon-btn"
                            title={t('settings.copyPath')}
                            aria-label={t('settings.copyPath')}
                            onClick={() => {
                                void navigator.clipboard
                                    .writeText(settings.backupPath)
                                    .then(() => pushToast(t('settings.copiedToast'), 'success'))
                                    .catch(() => pushToast(t('toasts.error'), 'error'));
                            }}
                        >
                            <IconLineCards size={18} className="settings-line-icon" />
                        </button>
                        <button
                            type="button"
                            className="icon-btn"
                            title={t('settings.browseBtn')}
                            aria-label={t('settings.browseBtn')}
                            onClick={() => void openBackupFolder()}
                        >
                            <IconLineFolder size={18} className="settings-line-icon" />
                        </button>
                    </div>
                </Row>
            </Section>

            <Section title={t('settings.extra')}>
                <Row
                    icon={<IconLineChart className="settings-line-icon" />}
                    label={t('settings.telemetry')}
                    sub={t('settings.telemetrySub')}
                >
                    <Toggle
                        checked={settings.telemetry}
                        onChange={(v) => update('telemetry', v)}
                        label={t('settings.telemetry')}
                    />
                </Row>
            </Section>
        </div>
    );
}
