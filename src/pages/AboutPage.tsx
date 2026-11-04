import { AppLogo } from '../components/Sidebar';
import {
  IconBug,
  IconExternal,
  IconGithub,
  IconHeart,
  IconLink,
  IconWindows,
} from '../components/icons';
import { IllustrationLaptopGear } from '../components/illustrations';
import { Button } from '../components/ui';
import { useApp } from '../store/app';

const REPO_URL = 'https://github.com/KirillGrachev/DriverUpdater';

export function AboutPage() {
  const { t, systemInfo } = useApp();

  return (
    <div className="page page-about">
      <section className="hero hero-left hero-compact">
        <div className="hero-text">
          <h1 className="hero-title">{t('about.title')}</h1>
          <p className="hero-sub">{t('about.subtitle')}</p>
        </div>
        <div className="hero-illu">
          <IllustrationLaptopGear />
        </div>
      </section>

      <section className="about-identity">
        <AppLogo size={116} />
        <div className="about-identity-text">
          <div className="about-name-row">
            <h2>Driver Updater</h2>
            <span className="version-pill">v1.0.0</span>
          </div>
          <p>
            {t('about.tagline1')}
            <br />
            {t('about.tagline2')}
          </p>
          <div className="about-links">
            <Button variant="outline" size="sm" icon={<IconGithub size={17} />} href={REPO_URL}>
              {t('about.github')}
            </Button>
            <Button variant="outline" size="sm" icon={<IconLink size={16} />} href={`${REPO_URL}/blob/main/README.md`}>
              {t('about.docs')}
            </Button>
            <Button variant="outline" size="sm" icon={<IconBug size={16} />} href={`${REPO_URL}/issues`}>
              {t('about.report')}
            </Button>
          </div>
        </div>
      </section>

      <section className="card about-info-card">
        <div className="about-info-col">
          <h3 className="card-title">{t('about.appInfo')}</h3>
          <dl className="about-rows">
            <div>
              <dt>{t('about.version')}</dt>
              <dd>1.0.0</dd>
            </div>
            <div>
              <dt>{t('about.build')}</dt>
              <dd>2025.11.02</dd>
            </div>
            <div>
              <dt>{t('about.license')}</dt>
              <dd>MIT License (Open Source)</dd>
            </div>
            <div>
              <dt>{t('about.developer')}</dt>
              <dd>{t('about.developerValue')}</dd>
            </div>
          </dl>
        </div>
        <div className="about-info-col">
          <span className="about-windows-logo">
            <IconWindows size={40} />
          </span>
          <h3 className="card-title">{t('about.sysInfo')}</h3>
          <dl className="about-rows">
            <div>
              <dt>{t('about.os')}</dt>
              <dd>{systemInfo?.os ?? '—'}</dd>
            </div>
            <div>
              <dt>{t('about.arch')}</dt>
              <dd>{systemInfo?.arch ?? '—'}</dd>
            </div>
            <div>
              <dt>{t('about.platform')}</dt>
              <dd>{systemInfo?.platform ?? '—'}</dd>
            </div>
          </dl>
        </div>
      </section>

      <section className="card about-thanks">
        <span className="about-thanks-icon">
          <IconHeart size={30} />
        </span>
        <div className="about-thanks-text">
          <h3>{t('about.thanks')}</h3>
          <p>
            {t('about.thanksSub1')}
            <br />
            {t('about.thanksSub2')}
          </p>
        </div>
        <Button variant="secondary" icon={<IconGithub size={17} />} href={`${REPO_URL}/issues/new`}>
          {t('about.review')}
        </Button>
      </section>

      <footer className="about-footer">
        <span>{t('about.footer')}</span>
        <a className="link-btn" href={`${REPO_URL}/blob/main/LICENSE`} target="_blank" rel="noreferrer noopener">
          {t('about.licenseLink')}
          <IconExternal size={15} />
        </a>
      </footer>
    </div>
  );
}
