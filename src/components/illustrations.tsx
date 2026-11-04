/**
 * Hero illustrations replicated from the designer mockups.
 * Pure inline SVG so the app stays fully offline (no CDN assets).
 */
import type { ReactNode } from 'react';

interface IllustrationProps {
  className?: string;
}

const Blobs = ({ id }: { id: string }) => (
  <g fill={`url(#${id})`} opacity="0.55">
    <circle cx="60" cy="70" r="52" />
    <circle cx="150" cy="34" r="40" />
    <circle cx="228" cy="78" r="58" />
    <circle cx="120" cy="120" r="46" />
  </g>
);

function Defs({ id }: { id: string }) {
  return (
    <defs>
      <linearGradient id={id} x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stopColor="var(--illu-blob-a)" />
        <stop offset="100%" stopColor="var(--illu-blob-b)" />
      </linearGradient>
      <linearGradient id={`${id}-screen`} x1="0" y1="0" x2="0.6" y2="1">
        <stop offset="0%" stopColor="var(--illu-screen-a)" />
        <stop offset="100%" stopColor="var(--illu-screen-b)" />
      </linearGradient>
      <linearGradient id={`${id}-blue`} x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stopColor="#4d94ff" />
        <stop offset="100%" stopColor="#1d6ef0" />
      </linearGradient>
    </defs>
  );
}

function Laptop({ id, children }: { id: string; children?: ReactNode }) {
  return (
    <g>
      <g transform="translate(52 26) skewX(-6)">
        <rect
          x="0"
          y="0"
          width="220"
          height="140"
          rx="10"
          fill={`url(#${id}-screen)`}
          stroke="var(--illu-stroke)"
          strokeWidth="5"
        />
        <g transform="translate(110 70)">{children}</g>
      </g>
      <path
        d="M28 168h268c4 0 6 3 5 6l-3 8c-1 3-4 5-7 5H35c-3 0-6-2-7-5l-3-8c-1-3 1-6 3-6z"
        fill="var(--illu-base)"
      />
      <rect x="128" y="168" width="68" height="7" rx="3.5" fill="var(--illu-base-dark)" />
    </g>
  );
}

export function IllustrationLaptopCheck({ className }: IllustrationProps) {
  const id = 'illu-check';
  return (
    <svg viewBox="0 0 340 200" className={className} aria-hidden="true">
      <Defs id={id} />
      <Blobs id={id} />
      <Laptop id={id}>
        <circle r="34" fill={`url(#${id}-blue)`} />
        <path
          d="m-14 0 9 9 19-20"
          stroke="#fff"
          strokeWidth="6"
          strokeLinecap="round"
          strokeLinejoin="round"
          fill="none"
        />
      </Laptop>
    </svg>
  );
}

export function IllustrationLaptopSync({ className }: IllustrationProps) {
  const id = 'illu-sync';
  return (
    <svg viewBox="0 0 340 200" className={className} aria-hidden="true">
      <Defs id={id} />
      <Blobs id={id} />
      <Laptop id={id}>
        <g stroke={`url(#${id}-blue)`} strokeWidth="9" strokeLinecap="round" fill="none">
          <path d="M-26 6a27 27 0 0 1 46-16" />
          <path d="M26-6a27 27 0 0 1-46 16" />
        </g>
        <path d="M22-22v14h-14" stroke={`url(#${id}-blue)`} strokeWidth="9" strokeLinecap="round" strokeLinejoin="round" fill="none" />
        <path d="M-22 22v-14h14" stroke={`url(#${id}-blue)`} strokeWidth="9" strokeLinecap="round" strokeLinejoin="round" fill="none" />
      </Laptop>
    </svg>
  );
}

export function IllustrationLaptopGear({ className }: IllustrationProps) {
  const id = 'illu-gear';
  return (
    <svg viewBox="0 0 340 200" className={className} aria-hidden="true">
      <Defs id={id} />
      <Blobs id={id} />
      <path
        d="M300 44l4 10 10 4-10 4-4 10-4-10-10-4 10-4z"
        fill={`url(#${id}-blue)`}
        opacity="0.8"
      />
      <Laptop id={id}>
        <g fill={`url(#${id}-blue)`}>
          <path d="M0-34c3 0 6 .4 8.8 1.1l2.4 8.9c3 1 5.8 2.6 8.2 4.6l8.8-2.6c4 4.4 7 9.6 8.6 15.4l-6.4 6.6c.5 3.1.5 6.3 0 9.4l6.4 6.6a36 36 0 0 1-8.6 15.4l-8.8-2.6c-2.4 2-5.2 3.6-8.2 4.6l-2.4 8.9a36 36 0 0 1-17.6 0l-2.4-8.9c-3-1-5.8-2.6-8.2-4.6l-8.8 2.6a36 36 0 0 1-8.6-15.4l6.4-6.6c-.5-3.1-.5-6.3 0-9.4l-6.4-6.6c1.6-5.8 4.6-11 8.6-15.4l8.8 2.6c2.4-2 5.2-3.6 8.2-4.6l2.4-8.9C-6-33.6-3-34 0-34z" />
          <circle r="12" fill="var(--illu-screen-a)" />
        </g>
      </Laptop>
    </svg>
  );
}

export function IllustrationDriveClock({ className }: IllustrationProps) {
  const id = 'illu-drive';
  return (
    <svg viewBox="0 0 340 200" className={className} aria-hidden="true">
      <Defs id={id} />
      <Blobs id={id} />
      <g transform="translate(96 40)">
        <path
          d="M18 18 40 0h96l22 18v84c0 6-5 10-10 10H28c-5 0-10-4-10-10z"
          fill="var(--illu-screen-b)"
          stroke="var(--illu-stroke)"
          strokeWidth="4"
        />
        <rect x="18" y="18" width="140" height="94" rx="10" fill="var(--illu-screen-a)" opacity="0.65" />
        <circle cx="46" cy="86" r="5" fill="var(--illu-stroke)" />
        <circle cx="66" cy="86" r="5" fill="var(--illu-stroke)" />
        <rect x="96" y="80" width="46" height="12" rx="6" fill="var(--illu-stroke)" opacity="0.5" />
      </g>
      <g transform="translate(216 96)">
        <circle r="42" fill="var(--illu-surface)" opacity="0.9" />
        <g stroke={`url(#${id}-blue)`} strokeWidth="10" strokeLinecap="round" fill="none">
          <path d="M-24-14a30 30 0 1 1-6 22" />
        </g>
        <path d="M-34-2v-16h16" stroke={`url(#${id}-blue)`} strokeWidth="10" strokeLinecap="round" strokeLinejoin="round" fill="none" />
        <path d="M0-16v16l11 8" stroke={`url(#${id}-blue)`} strokeWidth="9" strokeLinecap="round" strokeLinejoin="round" fill="none" />
      </g>
    </svg>
  );
}

export function IllustrationGearHero({ className }: IllustrationProps) {
  const id = 'illu-gehero';
  return (
    <svg viewBox="0 0 340 200" className={className} aria-hidden="true">
      <Defs id={id} />
      <Blobs id={id} />
      <g transform="translate(170 100)" fill={`url(#${id}-blue)`}>
        <path d="M0-62c5 0 10 .6 14.8 1.8l4 15c5 1.8 9.7 4.4 13.8 7.8l14.8-4.4a62 62 0 0 1 14.6 26l-10.8 11.2c.8 5.2.8 10.6 0 15.8L62 24.4a62 62 0 0 1-14.6 26l-14.8-4.4c-4.1 3.4-8.8 6-13.8 7.8l-4 15a62 62 0 0 1-29.6 0l-4-15c-5-1.8-9.7-4.4-13.8-7.8l-14.8 4.4a62 62 0 0 1-14.6-26l10.8-11.2c-.8-5.2-.8-10.6 0-15.8L-52-13.8a62 62 0 0 1 14.6-26l14.8 4.4c4.1-3.4 8.8-6 13.8-7.8l4-15C-10-61.4-5-62 0-62z" />
        <circle r="22" fill="var(--illu-surface)" />
      </g>
    </svg>
  );
}
