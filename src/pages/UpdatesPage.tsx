import { useMemo, useState } from 'react';
import {
  DriverGlyph,
  IconArrowRight,
  IconCheckCircleFilled,
  IconChevronDown,
  IconDownload,
  IconInfoFilled,
} from '../components/icons';
import { Badge, Button, Checkbox, Collapse } from '../components/ui';
import type { Driver } from '../ipc/types';
import { useApp } from '../store/app';
import { formatDate } from '../utils/format';

function UpdateRow({
  driver,
  checked,
  onToggle,
}: {
  driver: Driver;
  checked: boolean;
  onToggle: (next: boolean) => void;
}) {
  const { t, locale, installingIds } = useApp();
  const [expanded, setExpanded] = useState(false);
  const installing = installingIds.includes(driver.id);

  return (
    <li className={`update-row ${expanded ? 'expanded' : ''} ${checked ? 'selected' : ''}`}>
      <div className="update-row-main">
        <Checkbox
          checked={checked}
          disabled={installing}
          onChange={onToggle}
          label={driver.name}
        />
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
        <div className="update-size">
          ~ {driver.sizeMb} {t('common.mb')}
        </div>
        <div className="driver-badge">
          <Badge tone={driver.status === 'update' ? 'info' : 'neutral'}>
            {driver.status === 'update' ? t('updates.recommended') : t('updates.optional')}
          </Badge>
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
      <Collapse open={expanded}>
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
      </Collapse>
    </li>
  );
}

export function UpdatesPage() {
  const { t, data, installingIds, installUpdates, navigate } = useApp();
  const [selected, setSelected] = useState<Set<string> | null>(null);
  const [installing, setInstalling] = useState(false);

  const recommended = useMemo(
    () => (data?.drivers ?? []).filter((d) => d.status === 'update'),
    [data],
  );
  const optional = useMemo(
    () => (data?.drivers ?? []).filter((d) => d.status === 'optional'),
    [data],
  );

  if (!data) return null;

  // On first render everything recommended is preselected (as in the mockup).
  const effectiveSelected =
    selected ?? new Set<string>(recommended.map((d) => d.id));
  const allSelectable = [...recommended, ...optional];
  const selectedDrivers = allSelectable.filter((d) => effectiveSelected.has(d.id));
  const selectedRecommended = recommended.filter((d) => effectiveSelected.has(d.id));
  const totalSize = selectedDrivers.reduce((acc, d) => acc + d.sizeMb, 0);
  const allChecked = recommended.length > 0 && selectedRecommended.length === recommended.length;
  const someChecked = selectedRecommended.length > 0;
  const busy = installing || installingIds.length > 0;

  const toggle = (id: string, next: boolean) => {
    const copy = new Set(effectiveSelected);
    if (next) copy.add(id);
    else copy.delete(id);
    setSelected(copy);
  };

  const toggleAll = (next: boolean) => {
    setSelected(next ? new Set(recommended.map((d) => d.id)) : new Set());
  };

  const install = async () => {
    const ids = selectedDrivers.map((d) => d.id);
    setInstalling(true);
    setSelected(new Set());
    await installUpdates(ids);
    setInstalling(false);
  };

  const hasAny = allSelectable.length > 0;

  return (
    <div className="page page-updates">
      <section className="hero hero-left">
        <div className="hero-text">
          <h1 className="hero-title">{t('updates.title')}</h1>
          <p className="hero-sub">{hasAny ? t('updates.subtitle') : t('updates.emptySub')}</p>
        </div>
      </section>

      {hasAny ? (
        <>
          <section className="selection-bar">
            <div className="selection-info">
              <IconInfoFilled size={34} className="notice-icon-blue" />
              <div>
                <div className="selection-title">
                  {t('updates.selected', { n: selectedDrivers.length })}
                </div>
                <div className="selection-sub">
                  {t('updates.size', { n: totalSize })}
                </div>
              </div>
            </div>
            <Button
              size="lg"
              icon={<IconDownload size={18} />}
              disabled={selectedDrivers.length === 0}
              loading={busy}
              onClick={() => void install()}
            >
              {busy ? t('updates.installingBtn') : t('updates.installBtn')}
            </Button>
          </section>

          <section className="card updates-card">
            <div className="select-all-row">
              <Checkbox
                checked={allChecked}
                indeterminate={!allChecked && someChecked}
                onChange={toggleAll}
                label={t('updates.selectAll')}
              />
              <span className="select-all-label">{t('updates.selectAll')}</span>
            </div>
            <ul className="update-list">
              {recommended.map((driver) => (
                <UpdateRow
                  key={driver.id}
                  driver={driver}
                  checked={effectiveSelected.has(driver.id)}
                  onToggle={(next) => toggle(driver.id, next)}
                />
              ))}
            </ul>
            {optional.length > 0 ? (
              <>
                <div className="group-head group-head-plain">
                  <h2 className="card-title">{t('updates.other')}</h2>
                </div>
                <ul className="update-list">
                  {optional.map((driver) => (
                    <UpdateRow
                      key={driver.id}
                      driver={driver}
                      checked={effectiveSelected.has(driver.id)}
                      onToggle={(next) => toggle(driver.id, next)}
                    />
                  ))}
                </ul>
              </>
            ) : null}
          </section>

          <section className="notice-bar notice-info">
            <span className="notice-icon-green">
              <IconCheckCircleFilled size={26} />
            </span>
            <span className="notice-text">{t('updates.footerNote')}</span>
            <button type="button" className="link-btn" onClick={() => navigate('backups')}>
              {t('updates.footerMore')}
              <IconArrowRight size={16} />
            </button>
          </section>
        </>
      ) : (
        <section className="card empty-card">
          <IconCheckCircleFilled size={44} className="empty-icon-green" />
          <h2>{t('updates.emptyTitle')}</h2>
          <p>{t('updates.emptySub')}</p>
        </section>
      )}
    </div>
  );
}
