import { useMemo, useState } from 'react';
import {
  DriverGlyph,
  IconAlert,
  IconCheck,
  IconChevronDown,
  IconDatabase,
  IconRefresh,
  IconSearch,
  IconShield,
} from '../components/icons';
import { Badge, Button, Select, Segmented, StatCard } from '../components/ui';
import type { Driver, DriverCategory } from '../ipc/types';
import { useApp } from '../store/app';
import { formatDate, formatRelativeDay } from '../utils/format';

const CATEGORIES: DriverCategory[] = ['video', 'audio', 'chipset', 'network', 'input'];

function DriverRow({ driver }: { driver: Driver }) {
  const { t, locale, installingIds, installDriver } = useApp();
  const [expanded, setExpanded] = useState(false);
  const installing = installingIds.includes(driver.id);
  const needsUpdate = driver.status === 'update';
  const optional = driver.status === 'optional';

  return (
    <li className={`driver-row ${expanded ? 'expanded' : ''}`}>
      <div className="driver-row-main">
        <DriverGlyph kind={driver.icon.kind} color={driver.icon.color} />
        <div className="driver-ident">
          <div className="driver-name">{driver.name}</div>
          <div className="driver-category">{t(`drivers.categories.${driver.category}`)}</div>
        </div>
        <div className="driver-versions">
          <div>
            {t('drivers.cur')} <strong>{driver.current}</strong>
          </div>
          {driver.available ? (
            <div>
              {t('drivers.avail')} <strong>{driver.available}</strong>
            </div>
          ) : null}
        </div>
        <div className="driver-badge">
          {needsUpdate ? (
            <Badge tone="info">{t('drivers.badgeUpdate')}</Badge>
          ) : optional ? (
            <Badge tone="warn">{t('drivers.badgeOptional')}</Badge>
          ) : (
            <Badge tone="neutral">{t('drivers.badgeCurrent')}</Badge>
          )}
        </div>
        <div className="driver-action">
          {needsUpdate ? (
            <Button size="sm" loading={installing} onClick={() => void installDriver(driver.id)}>
              {installing ? t('drivers.btnUpdating') : t('drivers.btnUpdate')}
            </Button>
          ) : optional ? (
            <Button variant="secondary" size="sm" loading={installing} onClick={() => void installDriver(driver.id)}>
              {installing ? t('drivers.btnUpdating') : t('drivers.btnUpdate')}
            </Button>
          ) : (
            <Button variant="ghost" size="sm" disabled>
              {t('drivers.btnInstalled')}
            </Button>
          )}
        </div>
        <button
          type="button"
          className="driver-expand"
          aria-expanded={expanded}
          aria-label={t('drivers.detailsDesc')}
          onClick={() => setExpanded((v) => !v)}
        >
          <IconChevronDown size={16} className={expanded ? 'rotated' : ''} />
        </button>
      </div>
      {expanded ? (
        <div className="driver-details">
          <div className="driver-details-grid">
            <div>
              <span className="detail-label">{t('drivers.detailsDevice')}</span>
              <span className="detail-value">{driver.device}</span>
            </div>
            <div>
              <span className="detail-label">{t('drivers.detailsVendor')}</span>
              <span className="detail-value">{driver.vendor}</span>
            </div>
            <div>
              <span className="detail-label">{t('drivers.detailsReleased')}</span>
              <span className="detail-value">{formatDate(driver.released, locale)}</span>
            </div>
            <div>
              <span className="detail-label">{t('drivers.detailsSize')}</span>
              <span className="detail-value">
                ~ {driver.sizeMb} {t('common.mb')}
              </span>
            </div>
          </div>
          <p className="detail-desc">{driver.description[locale === 'ru' ? 'ru' : 'en']}</p>
        </div>
      ) : null}
    </li>
  );
}

export function DriversPage() {
  const { t, locale, data, scanning, scan, createRestorePoint } = useApp();
  const [query, setQuery] = useState('');
  const [category, setCategory] = useState<'all' | DriverCategory>('all');
  const [view, setView] = useState<'list' | 'categories'>('list');
  const [creatingPoint, setCreatingPoint] = useState(false);

  const drivers = useMemo(() => {
    if (!data) return [];
    const q = query.trim().toLowerCase();
    return data.drivers.filter((d) => {
      const matchesQuery =
        q.length === 0 ||
        d.name.toLowerCase().includes(q) ||
        d.vendor.toLowerCase().includes(q);
      const matchesCategory = category === 'all' || d.category === category;
      return matchesQuery && matchesCategory;
    });
  }, [data, query, category]);

  if (!data) return null;

  const needUpdate = data.drivers.filter((d) => d.status === 'update').length;
  const when = formatRelativeDay(data.summary.lastCheck, locale, t);

  const categoryOptions = [
    { value: 'all' as const, label: t('drivers.filterAll') },
    ...CATEGORIES.map((c) => ({ value: c, label: t(`drivers.categories.${c}`) })),
  ];

  const grouped = CATEGORIES.map((c) => ({
    category: c,
    items: drivers.filter((d) => d.category === c),
  })).filter((g) => g.items.length > 0);

  return (
    <div className="page page-drivers">
      <section className="page-head">
        <div>
          <h1 className="page-title">{t('drivers.title')}</h1>
          <p className="page-sub">{t('drivers.subtitle')}</p>
        </div>
        <label className="search-field">
          <IconSearch size={18} />
          <input
            type="search"
            value={query}
            placeholder={t('drivers.search')}
            onChange={(e) => setQuery(e.target.value)}
          />
        </label>
      </section>

      <section className="stats-grid stats-grid-4">
        <StatCard icon={<IconDatabase size={22} />} tone="blue" value={data.summary.devicesTotal} label={t('drivers.statDevices')} />
        <StatCard icon={<IconCheck size={22} />} tone="green" value={data.summary.upToDate} label={t('drivers.statCurrent')} />
        <StatCard icon={<IconAlert size={22} />} tone="orange" value={needUpdate} label={t('drivers.statNeed')} />
        <StatCard
          icon={<IconDatabase size={22} />}
          tone="blue"
          value={<span className="stat-value-sm">{when}</span>}
          label={t('drivers.statLast')}
          action={
            <button type="button" className="icon-btn" title={t('home.checkBtn')} onClick={() => void scan()}>
              <IconRefresh size={17} className={scanning ? 'spin' : ''} />
            </button>
          }
        />
      </section>

      <section className="drivers-toolbar">
        <Select
          value={category}
          onChange={setCategory}
          options={categoryOptions}
          ariaLabel={t('drivers.filterAll')}
        />
        <Segmented
          value={view}
          onChange={setView}
          ariaLabel={t('drivers.viewList')}
          options={[
            { value: 'list', label: t('drivers.viewList'), icon: <IconDatabase size={15} /> },
            { value: 'categories', label: t('drivers.viewCategories'), icon: <IconCheck size={15} /> },
          ]}
        />
      </section>

      {drivers.length === 0 ? (
        <section className="card empty-card">
          <h2>{t('drivers.empty')}</h2>
          <p>{t('drivers.emptySub')}</p>
        </section>
      ) : view === 'list' ? (
        <section className="card drivers-card">
          <ul className="driver-list">
            {drivers.map((driver) => (
              <DriverRow key={driver.id} driver={driver} />
            ))}
          </ul>
        </section>
      ) : (
        <section className="drivers-groups">
          {grouped.map((group) => (
            <div className="card drivers-card" key={group.category}>
              <div className="group-head">
                <h2 className="card-title">{t(`drivers.categories.${group.category}`)}</h2>
                <span className="group-count">{group.items.length}</span>
              </div>
              <ul className="driver-list">
                {group.items.map((driver) => (
                  <DriverRow key={driver.id} driver={driver} />
                ))}
              </ul>
            </div>
          ))}
        </section>
      )}

      <section className="notice-bar notice-info">
        <span className="notice-text">{t('drivers.footerNote')}</span>
        <Button
          variant="outline"
          size="sm"
          icon={<IconShield size={16} />}
          loading={creatingPoint}
          onClick={() => {
            setCreatingPoint(true);
            void createRestorePoint().finally(() => setCreatingPoint(false));
          }}
        >
          {creatingPoint ? t('backups.creatingBtn') : t('drivers.footerBtn')}
        </Button>
      </section>
    </div>
  );
}
