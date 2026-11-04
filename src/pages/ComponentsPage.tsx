import { useEffect, useState } from 'react';
import { IconCheck, IconDownload, IconPackage } from '../components/icons';
import { Button, Spinner } from '../components/ui';
import {
  COMPONENT_CATALOG,
  loadInstalledComponents,
  saveInstalledComponents,
  type AppComponent,
  type ComponentGroup,
} from '../data/components';
import type { InstalledComponent } from '../ipc/types';
import { useApp } from '../store/app';

const GROUPS: { id: ComponentGroup; titleKey: string }[] = [
  { id: 'runtime', titleKey: 'components.runtimes' },
  { id: 'language', titleKey: 'components.languages' },
  { id: 'tool', titleKey: 'components.tools' },
];

export function ComponentsPage() {
  const { t, pushToast, detectInstalledComponents } = useApp();
  const [installed, setInstalled] = useState<Set<string>>(loadInstalledComponents);
  const [installing, setInstalling] = useState<string[]>([]);
  const [detected, setDetected] = useState<Record<string, InstalledComponent>>({});
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void detectInstalledComponents()
      .then((list) => {
        if (cancelled || list.length === 0) return;
        setInstalled((prev) => {
          const next = new Set(prev);
          for (const component of list) next.add(component.id);
          saveInstalledComponents(next);
          return next;
        });
        setDetected(Object.fromEntries(list.map((component) => [component.id, component])));
      })
      .finally(() => {
        if (!cancelled) setReady(true);
      });
    return () => {
      cancelled = true;
    };
  }, [detectInstalledComponents]);

  const install = (item: AppComponent) => {
    if (installing.includes(item.id) || installed.has(item.id)) return;
    setInstalling((prev) => [...prev, item.id]);
    window.setTimeout(() => {
      setInstalled((prev) => {
        const next = new Set(prev);
        next.add(item.id);
        saveInstalledComponents(next);
        return next;
      });
      setInstalling((prev) => prev.filter((id) => id !== item.id));
      pushToast(t('components.installedToast', { name: item.name }), 'success');
    }, 900);
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
                const isInstalled = installed.has(item.id);
                const isInstalling = installing.includes(item.id);
                return (
                  <li className="component-row" key={item.id}>
                    <span className="component-icon">
                      <IconPackage size={20} />
                    </span>
                    <div className="component-ident">
                      <div className="component-name">{item.name}</div>
                      <div className="component-vendor">{item.vendor}</div>
                    </div>
                    <div className="component-version">
                      v{detected[item.id]?.version || item.version}
                    </div>
                    <div className="component-size">
                      ~ {item.sizeMb} {t('common.mb')}
                    </div>
                    {isInstalled ? (
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
                        onClick={() => install(item)}
                      >
                        {isInstalling ? t('components.installing') : t('components.install')}
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
