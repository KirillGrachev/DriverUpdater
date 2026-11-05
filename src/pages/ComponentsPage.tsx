import { useEffect, useState, Fragment } from 'react';
import { IconCheck, IconDownload, IconPackage } from '../components/icons';
import { Button, Select, Spinner } from '../components/ui';
import { COMPONENT_CATALOG, type AppComponent, type ComponentGroup, type ComponentSubgroup } from '../data/components';
import { PACKAGE_CANCELLED_ERROR, type InstalledComponent } from '../ipc/types';
import { useApp } from '../store/app';

const remoteVersionCache = new Map<string, string[]>();
const versionRequests = new Map<string, Promise<void>>();

const RUNTIME_SUBGROUPS: { id: ComponentSubgroup; titleKey: string }[] = [
    { id: 'dotnet', titleKey: 'components.groupDotnet' },
    { id: 'native', titleKey: 'components.groupNative' },
    { id: 'studio', titleKey: 'components.groupStudio' },
];

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
        cancelPackageInstall,
        packageInstalling,
        openModal,
        closeModal,
    } = useApp();
    const [installed, setInstalled] = useState<Set<string>>(() => new Set());
    const [installedVersions, setInstalledVersions] = useState<Record<string, string>>({});
    const [selected, setSelected] = useState<Record<string, string>>(defaultSelection);
    const [versions, setVersions] = useState<Record<string, string[]>>(defaultVersions);
    const [detected, setDetected] = useState<Record<string, InstalledComponent>>({});
    const [ready, setReady] = useState(false);

    useEffect(() => {
        let cancelled = false;
        void Promise.all([detectInstalledComponents(), getInstalledComponents()])
            .then(([detectedComponents, persistedIds]) => {
                if (cancelled) return;
                const next = new Set<string>();
                const nextSelected = defaultSelection();
                const nextVersions: Record<string, string> = {};
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

    const loadVersions = (item: AppComponent) => {
        if (!item.wingetId) return;
        const cached = remoteVersionCache.get(item.id);
        if (cached) {
            setVersions((prev) => (prev[item.id] === cached ? prev : { ...prev, [item.id]: cached }));
            return;
        }
        if (versionRequests.has(item.id)) return;
        const request = listPackageVersions(item.wingetId)
            .then((remote) => {
                if (remote.length === 0) return;
                remoteVersionCache.set(item.id, remote);
                setVersions((prev) => ({ ...prev, [item.id]: remote }));
            })
            .finally(() => {
                versionRequests.delete(item.id);
            });
        versionRequests.set(item.id, request);
    };

    const install = async (item: AppComponent) => {
        if (!item.wingetId || packageInstalling.includes(item.wingetId)) return;
        const version = selected[item.id] || item.defaultVersion;
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
        } catch (error) {
            const message = typeof error === 'string' ? error : error instanceof Error ? error.message : '';
            if (message === PACKAGE_CANCELLED_ERROR) {
                pushToast(t('toasts.error'), 'info');
                return;
            }
            console.error('component install failed', error);
            pushToast(t('toasts.error'), 'error');
        }
    };

    const requestInstall = (item: AppComponent) => {
        if ((item.sizeMb ?? 0) >= 1000) {
            openModal({
                title: t('components.confirmLargeTitle'),
                text: t('components.confirmLargeText', { name: item.name, size: item.sizeMb ?? 0 }),
                confirmLabel: t('components.install'),
                cancelLabel: t('backups.confirmCancel'),
                tone: 'primary',
                onConfirm: async () => {
                    closeModal();
                    await install(item);
                },
            });
            return;
        }
        void install(item);
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
                                {(() => {
                                    let lastSubgroup: ComponentSubgroup | undefined;
                                    return items.map((item) => {
                                        const showSubgroup =
                                            group.id === 'runtime' &&
                                            item.subgroup !== undefined &&
                                            item.subgroup !== lastSubgroup;
                                        lastSubgroup = item.subgroup;
                                        const detectedVersion = detected[item.id]?.version;
                                        const installedVersion = detectedVersion || installedVersions[item.id];
                                        const value = selected[item.id] || installedVersion || item.defaultVersion;
                                        const isInstalled = installed.has(item.id);
                                        const isInstalling = packageInstalling.includes(item.wingetId ?? '');
                                        const options = versions[item.id] ?? item.versions;
                                        return (
                                            <Fragment key={item.id}>
                                                {showSubgroup ? (
                                                    <li className="component-subhead">
                                                        {t(
                                                            RUNTIME_SUBGROUPS.find(
                                                                (subgroup) => subgroup.id === item.subgroup,
                                                            )?.titleKey ?? '',
                                                        )}
                                                    </li>
                                                ) : null}
                                                <li className="component-row">
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
                                                            ariaLabel={`${item.name} ${t('components.version')}`}
                                                            onOpen={() => loadVersions(item)}
                                                            options={[value, detectedVersion, ...options]
                                                                .filter(
                                                                    (version, index, list): version is string =>
                                                                        Boolean(version) &&
                                                                        list.indexOf(version) === index,
                                                                )
                                                                .map((version) => ({
                                                                    value: version,
                                                                    label: `v${version}`,
                                                                }))}
                                                        />
                                                    )}
                                                    <div className="component-size">
                                                        {item.sizeMb
                                                            ? `~ ${item.sizeMb} ${t('common.mb')}`
                                                            : t('common.never')}
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
                                                    ) : isInstalling ? (
                                                        <Button
                                                            variant="secondary"
                                                            size="sm"
                                                            onClick={() => {
                                                                void cancelPackageInstall(item.wingetId ?? '');
                                                            }}
                                                        >
                                                            {t('backups.confirmCancel')}
                                                        </Button>
                                                    ) : (
                                                        <Button
                                                            variant="outline"
                                                            size="sm"
                                                            icon={<IconDownload size={15} />}
                                                            onClick={() => requestInstall(item)}
                                                        >
                                                            {t('components.install')}
                                                        </Button>
                                                    )}
                                                </li>
                                            </Fragment>
                                        );
                                    });
                                })()}
                            </ul>
                        </section>
                    );
                })
            )}
        </div>
    );
}
