import type { ReactNode } from 'react';
import { IconSettingsClock, IconSettingsPalette, IconSettingsSync, IconShield } from '../components/icons';
import { Select, Segmented, Toggle } from '../components/ui';
import { LocaleFlag } from '../components/Flags';
import sunIcon from '../assets/icons/icon-sun.svg';
import moonIcon from '../assets/icons/icon-moon.svg';
import systemIcon from '../assets/icons/icon-sun-moon.svg';
import globeIcon from '../assets/icons/icon-globe.svg';
import globeDarkIcon from '../assets/icons/icon-globe-dark.svg';
import cardsIcon from '../assets/icons/icon-cards.svg';
import cardsDarkIcon from '../assets/icons/icon-cards-dark.svg';
import databaseIcon from '../assets/icons/icon-database.svg';
import databaseDarkIcon from '../assets/icons/icon-database-dark.svg';
import folderIcon from '../assets/icons/icon-folder.svg';
import folderDarkIcon from '../assets/icons/icon-folder-dark.svg';
import { availableLocales } from '../i18n';
import type { CheckInterval, Settings, ThemeMode } from '../ipc/types';
import { useApp } from '../store/app';

function ThemedIcon({ light, dark }: { light: string; dark: string }) {
    return (
        <>
            <img className="settings-asset-icon theme-light-only" src={light} alt="" />
            <img className="settings-asset-icon theme-dark-only" src={dark} alt="" />
        </>
    );
}

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
    const displayBackupPath = settings.backupPath.replace(
        /^[A-Za-z]:\\Users\\[^\\]+\\AppData\\Local/i,
        '%LOCALAPPDATA%',
    );
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
                <Row icon={<ThemedIcon light={globeIcon} dark={globeDarkIcon} />} label={t('settings.language')}>
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
                <Row icon={<IconSettingsPalette size={21} />} label={t('settings.theme')}>
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
            </Section>

            <Section title={t('settings.secUpdates')}>
                <Row
                    icon={<IconSettingsSync size={21} />}
                    label={t('settings.autoCheck')}
                    sub={t('settings.autoCheckSub')}
                >
                    <Toggle
                        checked={settings.autoCheck}
                        onChange={(v) => update('autoCheck', v)}
                        label={t('settings.autoCheck')}
                    />
                </Row>
                <Row icon={<IconSettingsClock size={21} />} label={t('settings.interval')}>
                    <Select<CheckInterval>
                        className="interval-select"
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
                <Row icon={<IconShield size={21} strokeWidth={2} />} label={t('settings.restoreBefore')}>
                    <Toggle
                        checked={settings.restorePointBeforeInstall}
                        onChange={(v) => update('restorePointBeforeInstall', v)}
                        label={t('settings.restoreBefore')}
                    />
                </Row>
                <Row
                    icon={<ThemedIcon light={databaseIcon} dark={databaseDarkIcon} />}
                    label={t('settings.keepBackups')}
                >
                    <Toggle
                        checked={settings.keepBackups}
                        onChange={(v) => update('keepBackups', v)}
                        label={t('settings.keepBackups')}
                    />
                </Row>
                <Row icon={<ThemedIcon light={folderIcon} dark={folderDarkIcon} />} label={t('settings.backupPath')}>
                    <div className="path-control">
                        <div className="path-readonly path-value" title={settings.backupPath}>
                            {displayBackupPath}
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
                            <ThemedIcon light={cardsIcon} dark={cardsDarkIcon} />
                        </button>
                        <button
                            type="button"
                            className="icon-btn"
                            title={t('settings.browseBtn')}
                            aria-label={t('settings.browseBtn')}
                            onClick={() => void openBackupFolder()}
                        >
                            <ThemedIcon light={folderIcon} dark={folderDarkIcon} />
                        </button>
                    </div>
                </Row>
            </Section>
        </div>
    );
}
