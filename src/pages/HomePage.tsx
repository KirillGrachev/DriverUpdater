import { DriverGlyph, IconArrowRight, IconCheck, IconClock, IconDownload, IconMonitor, IconRefresh } from '../components/icons';
import { Button, StatCard } from '../components/ui';
import { formatRelativeDay } from '../utils/format';
import { useApp } from '../store/app';

// Home mockup: large stat tiles, a compact last-check card and tight rows.
export function HomePage() {
  const { t, locale, data, scanning, installingIds, navigate, scan, installDriver } = useApp();
  if (!data) return null;

  const recommended = data.drivers.filter((d) => d.status === 'update').slice(0, 3);
  const hasUpdates = data.summary.updatesAvailable > 0;
  const when = formatRelativeDay(data.summary.lastCheck, locale, t);

  return (
    <div className="page page-home">
      <section className="hero">
        <div className="hero-text">
          <h1 className="hero-title">
            {hasUpdates ? (
              <>
                {t('home.titleUpd1')}
                <br />
                {t('home.titleUpd2', { n: data.summary.updatesAvailable })}
              </>
            ) : (
              <>
                {t('home.titleOk1')}
                <br />
                {t('home.titleOk2')}
              </>
            )}
          </h1>
          <p className="hero-sub">
            {hasUpdates
              ? t('home.subUpdates', { n: data.summary.updatesAvailable, when })
              : t('home.subOk', { when })}
          </p>
        </div>
      </section>

      <section className="stats-grid stats-grid-3">
        <StatCard icon={<IconMonitor size={30} />} tone="blue" value={data.summary.devicesTotal} label={t('home.statDevices')} />
        <StatCard icon={<IconCheck size={30} />} tone="green" value={data.summary.upToDate} label={t('home.statCurrent')} />
        <StatCard icon={<IconDownload size={30} />} tone={hasUpdates ? 'orange' : 'blue'} value={data.summary.updatesAvailable} label={t('home.statUpdates')} />
      </section>

      <section className="card last-check-card">
        <div className="last-check-info">
          <h2 className="card-title">{t('home.lastCheckTitle')}</h2>
          <div className="last-check-row">
            <span className="last-check-icon">
              <IconClock size={22} />
            </span>
            <div>
              <div className="last-check-when">{data.summary.lastCheck ? when : t('home.lastCheckNever')}</div>
              <div className="last-check-sub">
                {t('home.lastCheckSub', { count: data.summary.devicesTotal })}
              </div>
            </div>
          </div>
        </div>
        <Button icon={<IconRefresh size={18} />} loading={scanning} onClick={() => void scan()}>
          {scanning ? t('home.checkingBtn') : t('home.checkBtn')}
        </Button>
      </section>

      <section className="card rec-card">
        <div className="card-head">
          <h2 className="card-title">{t('home.recTitle')}</h2>
          <button type="button" className="link-btn" onClick={() => navigate('drivers')}>
            {t('home.showAll')}
            <IconArrowRight size={16} />
          </button>
        </div>
        {recommended.length === 0 ? (
          <p className="rec-empty">{t('home.emptyRec')}</p>
        ) : (
          <ul className="rec-list">
            {recommended.map((driver) => (
              <li key={driver.id} className="rec-row">
                <DriverGlyph kind={driver.icon.kind} color={driver.icon.color} size={24} />
                <span className="rec-name">{driver.name}</span>
                <span className="rec-vendor">{driver.vendor}</span>
                <span className="rec-versions">
                  {driver.current} <span className="rec-arrow">→</span>{' '}
                  <strong>{driver.available}</strong>
                </span>
                <Button
                  variant="secondary"
                  size="sm"
                  className="rec-btn"
                  loading={installingIds.includes(driver.id)}
                  onClick={() => void installDriver(driver.id)}
                >
                  {installingIds.includes(driver.id) ? t('home.updatingBtn') : t('home.updateBtn')}
                </Button>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
