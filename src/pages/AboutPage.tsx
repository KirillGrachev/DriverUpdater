import logo from '../assets/branding/logo.png';
import aboutMascot from '../assets/branding/about-mascot.png';
import {
  IconBug,
  IconGithub,
  IconLink,
  IconWindows,
} from '../components/icons';
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
      </section>

      <section className="about-identity">
        <img className="about-logo" src={logo} alt="Driver Updater" draggable={false} />
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
              <dd>2026.11.04</dd>
            </div>
            <div>
              <dt>{t('about.license')}</dt>
              <dd>
                <a
                  className="dev-link"
                  href={`${REPO_URL}/blob/main/LICENSE`}
                  target="_blank"
                  rel="noreferrer noopener"
                >
                  MIT License (Open Source)
                </a>
              </dd>
            </div>
            <div>
              <dt>{t('about.developer')}</dt>
              <dd>
                <a
                  className="dev-link"
                  href="https://github.com/KirillGrachev"
                  target="_blank"
                  rel="noreferrer noopener"
                >
                  Kirill Grachev
                </a>
              </dd>
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
              <dd>{systemInfo?.os ?? t('common.never')}</dd>
            </div>
            <div>
              <dt>{t('about.computer')}</dt>
              <dd>{systemInfo?.hostname ?? t('common.never')}</dd>
            </div>
            <div>
              <dt>{t('about.arch')}</dt>
              <dd>{systemInfo?.arch ?? t('common.never')}</dd>
            </div>
            <div>
              <dt>{t('about.platform')}</dt>
              <dd>{systemInfo?.platform ?? t('common.never')}</dd>
            </div>
          </dl>
        </div>
      </section>

      <section className="card about-thanks">
        <img className="about-thanks-mascot" src={aboutMascot} alt="" draggable={false} />
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
      </footer>
    </div>
  );
}
