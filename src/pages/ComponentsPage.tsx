import { useEffect, useRef, useState } from 'react';
import { IconCheck, IconDownload, IconPackage } from '../components/icons';
import { Button, Select, Spinner } from '../components/ui';
import { COMPONENT_CATALOG, type AppComponent, type ComponentGroup } from '../data/components';
import type { InstalledComponent } from '../ipc/types';
import { useApp } from '../store/app';

const GROUPS: { id: ComponentGroup; titleKey: string }[] = [
    { id: 'runtime', titleKey: 'components.runtimes' },
    { id: 'language', titleKey: 'components.languages' },
    { id: 'tool', titleKey: 'components.tools' },
];

function defaultSelection(): Record<string, string> {
    return Object.fromEntries(COMPONENT_CATALOG.map((item) => [item.id, item.defaultVersion]));
}

function defaultVersions(): Record<string, string[]> {
    return Object.fromEntries(COMPONENT_CATALOG.map((item) => [item.id, item.versions]));
}

export function ComponentsPage() {
    const {
        t,
        pushToast,
        detectInstalledComponents,
        getInstalledComponents,
        saveInstalledComponents,
        listPackageVersions,
        installPackage,
    } = useApp();
    const [installed, setInstalled] = useState<Set<string>>(
        () => new Set(COMPONENT_CATALOG.filter((item) => item.installedByDefault).map((item) => item.id)),
    );
    const [installedVersions, setInstalledVersions] = useState<Record<string, string>>(() =>
        Object.fromEntries(
            COMPONENT_CATALOG.filter((item) => item.installedByDefault).map((item) => [item.id, item.defaultVersion]),
        ),
    );
    const [selected, setSelected] = useState<Record<string, string>>(defaultSelection);
    const [versions, setVersions] = useState<Record<string, string[]>>(defaultVersions);
    const [loadingVersionIds, setLoadingVersionIds] = useState<Set<string>>(new Set());
    const [installing, setInstalling] = useState<string[]>([]);
    const [detected, setDetected] = useState<Record<string, InstalledComponent>>({});
    const [ready, setReady] = useState(false);
    const loadedVersions = useRef<Set<string>>(new Set());

    useEffect(() => {
        let cancelled = false;
        void Promise.all([detectInstalledComponents(), getInstalledComponents()])
            .then(([detectedComponents, persistedIds]) => {
                if (cancelled) return;
                const next = new Set(
                    COMPONENT_CATALOG.filter((item) => item.installedByDefault).map((item) => item.id),
                );
                const nextSelected = defaultSelection();
                const nextVersions: Record<string, string> = {
                    ...Object.fromEntries(
                        COMPONENT_CATALOG.filter((item) => item.installedByDefault).map((item) => [
                            item.id,
                            item.defaultVersion,
                        ]),
                    ),
                };
                for (const id of persistedIds) next.add(id);
                for (const component of detectedComponents) {
                    next.add(component.id);
                    nextVersions[component.id] = component.version;
                    if (component.id in nextSelected) {
                        nextSelected[component.id] = component.version;
                    }
                }
                setInstalled(next);
                setInstalledVersions(nextVersions);
                setSelected(nextSelected);
                setDetected(Object.fromEntries(detectedComponents.map((component) => [component.id, component])));
                if (detectedComponents.length > 0) {
                    void saveInstalledComponents([...next]);
                }
            })
            .finally(() => {
                if (!cancelled) setReady(true);
            });
        return () => {
            cancelled = true;
        };
    }, [detectInstalledComponents, getInstalledComponents, saveInstalledComponents]);

    const loadVersions = async (item: AppComponent) => {
        if (!item.wingetId || loadedVersions.current.has(item.id)) return;
        loadedVersions.current.add(item.id);
        setLoadingVersionIds((prev) => new Set(prev).add(item.id));
        try {
            const remote = await listPackageVersions(item.wingetId);
            if (remote.length === 0) return;
            setVersions((prev) => ({ ...prev, [item.id]: remote }));
            setSelected((prev) => ({
                ...prev,
                [item.id]: detected[item.id]?.version || remote[0],
            }));
        } finally {
            setLoadingVersionIds((prev) => {
                const next = new Set(prev);
                next.delete(item.id);
                return next;
            });
        }
    };

    const install = async (item: AppComponent) => {
        if (!item.wingetId || installing.includes(item.id)) return;
        const version = selected[item.id] || item.defaultVersion;
        setInstalling((prev) => [...prev, item.id]);
        try {
            await installPackage(item.wingetId, version);
            const next = new Set(installed);
            next.add(item.id);
            setInstalled(next);
            setInstalledVersions((prev) => ({ ...prev, [item.id]: version }));
            await saveInstalledComponents([...next]);
            const refreshed = await detectInstalledComponents(true);
            if (refreshed.length > 0) {
                setDetected(Object.fromEntries(refreshed.map((component) => [component.id, component])));
            }
            pushToast(t('components.installedToast', { name: item.name }), 'success');
        } catch {
            pushToast(t('toasts.error'), 'error');
        } finally {
            setInstalling((prev) => prev.filter((id) => id !== item.id));
        }
    };

    return (
        <div className="page page-components">
            <section className="hero hero-left hero-compact">
                <div className="hero-text">
                    <h1 className="hero-title">{t('components.title')}</h1>
                    <p className="hero-sub">{t('components.subtitle')}</p>
                </div>
            </section>

            {!ready ? (
                <section className="card components-checking">
                    <Spinner size={24} />
                    <span>{t('components.checking')}</span>
                </section>
            ) : (
                GROUPS.map((group) => {
                    const items = COMPONENT_CATALOG.filter((item) => item.group === group.id);
                    return (
                        <section className="card components-card" key={group.id}>
                            <div className="components-head">
                                <h2 className="card-title">{t(group.titleKey)}</h2>
                                <span className="group-count">{items.length}</span>
                            </div>
                            <ul className="component-list">
                                {items.map((item) => {
                                    const detectedVersion = detected[item.id]?.version;
                                    const installedVersion = detectedVersion || installedVersions[item.id];
                                    const value = selected[item.id] || installedVersion || item.defaultVersion;
                                    const isInstalled = installed.has(item.id);
                                    const isInstalling = installing.includes(item.id);
                                    const versionLoading = loadingVersionIds.has(item.id);
                                    const options = versions[item.id] ?? item.versions;
                                    return (
                                        <li className="component-row" key={item.id}>
                                            <span className="component-icon">
                                                <IconPackage size={20} />
                                            </span>
                                            <div className="component-ident">
                                                <div className="component-name">{item.name}</div>
                                                <div className="component-vendor">{item.vendor}</div>
                                            </div>
                                            {item.noteKey ? (
                                                <div className="component-version component-note">
                                                    {t(item.noteKey)}
                                                </div>
                                            ) : (
                                                <Select
                                                    className="component-version-select"
                                                    value={value}
                                                    onChange={(next) =>
                                                        setSelected((prev) => ({ ...prev, [item.id]: next }))
                                                    }
                                                    onOpen={() => void loadVersions(item)}
                                                    ariaLabel={`${item.name} ${t('components.version')}`}
                                                    options={[
                                                        ...(detectedVersion && !options.includes(detectedVersion)
                                                            ? [detectedVersion]
                                                            : []),
                                                        ...options,
                                                    ].map((version) => ({
                                                        value: version,
                                                        label: versionLoading ? t('components.loading') : `v${version}`,
                                                    }))}
                                                />
                                            )}
                                            <div className="component-size">
                                                {item.sizeMb ? `~ ${item.sizeMb} ${t('common.mb')}` : t('common.never')}
                                            </div>
                                            {!item.wingetId ? (
                                                <Button variant="secondary" size="sm" disabled>
                                                    {t('components.managed')}
                                                </Button>
                                            ) : isInstalled && value === installedVersion ? (
                                                <Button
                                                    variant="secondary"
                                                    size="sm"
                                                    icon={<IconCheck size={15} />}
                                                    disabled
                                                    title={detected[item.id]?.path || undefined}
                                                >
                                                    {t('components.installed')}
                                                </Button>
                                            ) : (
                                                <Button
                                                    variant="outline"
                                                    size="sm"
                                                    icon={<IconDownload size={15} />}
                                                    loading={isInstalling}
                                                    onClick={() => void install(item)}
                                                >
                                                    {isInstalling
                                                        ? t('components.installing')
                                                        : t('components.install')}
                                                </Button>
                                            )}
                                        </li>
                                    );
                                })}
                            </ul>
                        </section>
                    );
                })
            )}
        </div>
    );
}
