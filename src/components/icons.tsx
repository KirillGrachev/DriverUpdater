import type { ReactNode } from 'react';
import type { DriverIconKind } from '../ipc/types';

export interface IconProps {
    size?: number;
    className?: string;
    strokeWidth?: number;
}

function Svg({
    children,
    size = 20,
    className,
    strokeWidth = 2,
    fill = 'none',
}: IconProps & { children: ReactNode; fill?: string }) {
    return (
        <svg
            width={size}
            height={size}
            viewBox="0 0 24 24"
            fill={fill}
            stroke="currentColor"
            strokeWidth={strokeWidth}
            strokeLinecap="round"
            strokeLinejoin="round"
            className={className}
            aria-hidden="true"
            focusable="false"
        >
            {children}
        </svg>
    );
}

/** Navigation and window chrome icons. */

export const IconHome = (p: IconProps) => (
    <Svg {...p}>
        <path d="m3 10.5 9-7.5 9 7.5" />
        <path d="M5 9.5V21h14V9.5" />
        <path d="M10 21v-6h4v6" />
    </Svg>
);

export const IconDrivers = (p: IconProps) => (
    <Svg {...p}>
        <rect x="3" y="4" width="7" height="7" rx="1.5" />
        <rect x="3" y="14" width="7" height="6" rx="1.5" />
        <path d="M14 6h7M14 9h4M14 16h7M14 19h4" />
    </Svg>
);

export const IconUpdates = (p: IconProps) => (
    <Svg {...p}>
        <path d="M3 12a9 9 0 1 0 2.6-6.3L3 8" />
        <path d="M3 3v5h5" />
        <path d="M12 8v4l2.8 2.8" />
    </Svg>
);

export const IconBackups = (p: IconProps) => (
    <Svg {...p}>
        <path d="M12 3v8" />
        <path d="m8.5 7.5 3.5 3.5 3.5-3.5" />
        <path d="M4 13v5a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-5" />
    </Svg>
);

export const IconSettings = (p: IconProps) => (
    <Svg {...p}>
        <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
        <circle cx="12" cy="12" r="3" />
    </Svg>
);

export const IconInfo = (p: IconProps) => (
    <Svg {...p}>
        <circle cx="12" cy="12" r="9" />
        <path d="M12 11v5" />
        <path d="M12 8h.01" />
    </Svg>
);

export const IconInfoFilled = ({ size = 20, className }: IconProps) => (
    <svg
        width={(size * 574) / 580}
        height={size}
        viewBox="-24 -24 574 580"
        fill="none"
        className={className}
        aria-hidden="true"
        focusable="false"
    >
        <path
            d="M355.2 0H170.4C76.29 0 0 76.29 0 170.4V361.2C0 455.31 76.29 531.6 170.4 531.6H355.2C449.31 531.6 525.6 455.31 525.6 361.2V170.4C525.6 76.29 449.31 0 355.2 0Z"
            fill="var(--info-outer, var(--c-primary))"
        />
        <path
            d="M351.6 40.8H174C99.77 40.8 39.6 100.97 39.6 175.2V356.4C39.6 430.63 99.77 490.8 174 490.8H351.6C425.83 490.8 486 430.63 486 356.4V175.2C486 100.97 425.83 40.8 351.6 40.8Z"
            fill="var(--info-inner, var(--c-card))"
        />
        <circle cx="262.8" cy="158.4" r="44.4" fill="var(--info-glyph, var(--c-primary))" />
        <path
            d="M303.6 265.2C303.6 242.67 285.33 224.4 262.8 224.4C240.27 224.4 222 242.67 222 265.2V385.2C222 407.73 240.27 426 262.8 426C285.33 426 303.6 407.73 303.6 385.2V265.2Z"
            fill="var(--info-glyph, var(--c-primary))"
        />
    </svg>
);

/** Monochrome info icon for the inactive About button. */
export const IconSearch = (p: IconProps) => (
    <Svg {...p}>
        <circle cx="11" cy="11" r="7" />
        <path d="m20.5 20.5-4-4" />
    </Svg>
);

export const IconClose = (p: IconProps) => (
    <Svg {...p}>
        <path d="m6 6 12 12M18 6 6 18" />
    </Svg>
);

export const IconMinimize = (p: IconProps) => (
    <Svg {...p}>
        <path d="M5 12h14" />
    </Svg>
);

export const IconMaximize = (p: IconProps) => (
    <Svg {...p}>
        <rect x="6" y="6" width="12" height="12" rx="1.5" />
    </Svg>
);

export const IconChevronDown = (p: IconProps) => (
    <Svg {...p}>
        <path d="m6 9 6 6 6-6" />
    </Svg>
);

export const IconList = (p: IconProps) => (
    <Svg {...p}>
        <path d="M9 6h11M9 12h11M9 18h11" />
        <path d="M4 6h.01M4 12h.01M4 18h.01" strokeWidth={2.6} />
    </Svg>
);

export const IconGrid = (p: IconProps) => (
    <Svg {...p}>
        <rect x="3.5" y="3.5" width="7" height="7" rx="1.6" />
        <rect x="13.5" y="3.5" width="7" height="7" rx="1.6" />
        <rect x="3.5" y="13.5" width="7" height="7" rx="1.6" />
        <rect x="13.5" y="13.5" width="7" height="7" rx="1.6" />
    </Svg>
);

export const IconArrowRight = (p: IconProps) => (
    <Svg {...p}>
        <path d="M4 12h16" />
        <path d="m14 6 6 6-6 6" />
    </Svg>
);

export const IconExternal = (p: IconProps) => (
    <Svg {...p}>
        <path d="M15 3h6v6" />
        <path d="M10 14 21 3" />
        <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
    </Svg>
);

/** Statistics and state icons. */

export const IconMonitor = (p: IconProps) => (
    <Svg {...p}>
        <rect x="2" y="3" width="20" height="14" rx="2" />
        <path d="M8 21h8M12 17v4" />
    </Svg>
);

export const IconCheck = (p: IconProps) => (
    <Svg {...p}>
        <path d="m4.5 12.5 5 5 10-11" />
    </Svg>
);

export const IconCheckCircle = (p: IconProps) => (
    <Svg {...p}>
        <circle cx="12" cy="12" r="9" />
        <path d="m8.3 12.4 2.6 2.6 4.8-5.6" />
    </Svg>
);

export const IconCheckCircleFilled = ({ size = 20, className }: IconProps) => (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className} aria-hidden="true">
        <circle cx="12" cy="12" r="10" fill="currentColor" />
        <path
            d="m7.6 12.4 2.8 2.8 5.6-6.4"
            stroke="#fff"
            strokeWidth="2.2"
            strokeLinecap="round"
            strokeLinejoin="round"
            fill="none"
        />
    </svg>
);

export const IconAlert = (p: IconProps) => (
    <Svg {...p}>
        <circle cx="12" cy="12" r="9" />
        <path d="M12 7.5V13" />
        <path d="M12 16.5h.01" />
    </Svg>
);

export const IconDatabase = (p: IconProps) => (
    <Svg {...p}>
        <ellipse cx="12" cy="5.5" rx="8" ry="3" />
        <path d="M4 5.5v6c0 1.66 3.58 3 8 3s8-1.34 8-3v-6" />
        <path d="M4 11.5v6c0 1.66 3.58 3 8 3s8-1.34 8-3v-6" />
    </Svg>
);

export const IconRefresh = (p: IconProps) => (
    <Svg {...p}>
        <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
        <path d="M21 3v5h-5" />
        <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
        <path d="M8 16H3v5" />
    </Svg>
);

export const IconRestore = (p: IconProps) => (
    <Svg {...p}>
        <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
        <path d="M3 3v5h5" />
    </Svg>
);

/** Custom restore arrow used on the action button, not on list rows. */
export const IconRestoreArrow = ({ size = 20, className }: IconProps) => (
    <svg
        width={size}
        height={size}
        viewBox="0 0 430 427"
        fill="none"
        stroke="currentColor"
        strokeWidth={64}
        strokeLinecap="round"
        strokeLinejoin="round"
        className={className}
        aria-hidden="true"
        focusable="false"
    >
        <path d="M137 32L32 135L137 239" />
        <path d="M47 135H267C356 135 398 194 398 264C398 339 335 395 257 395H184" />
    </svg>
);

export const IconDownload = (p: IconProps) => (
    <Svg {...p}>
        <path d="M12 3v11" />
        <path d="m7.5 10.5 4.5 4.5 4.5-4.5" />
        <path d="M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" />
    </Svg>
);

export const IconClock = (p: IconProps) => (
    <Svg {...p}>
        <circle cx="12" cy="12" r="9" />
        <path d="M12 7v5l3.2 3.2" />
    </Svg>
);

export const IconCalendar = ({ size = 20, className }: IconProps) => (
    <svg
        width={(size * 573) / 542}
        height={size}
        viewBox="0 0 573 542"
        fill="none"
        className={className}
        aria-hidden="true"
        focusable="false"
    >
        <path
            d="M466.2 58.8H106.2C49.2 58.8 3 105 3 162V435.6C3 492.6 49.2 538.8 106.2 538.8H466.2C523.2 538.8 569.4 492.6 569.4 435.6V162C569.4 105 523.2 58.8 466.2 58.8Z"
            fill="var(--calendar-body)"
            stroke="var(--calendar-border)"
            strokeWidth="8"
        />
        <path
            d="M108.03 59H463.97C519.73 59 566 104.4 566 160.55V194H6V161.74C6 105.59 52.27 59 108.03 59Z"
            fill="var(--calendar-header)"
        />
        <path
            d="M207 35.4C207 15.85 191.15 0 171.6 0C152.05 0 136.2 15.85 136.2 35.4V111C136.2 130.55 152.05 146.4 171.6 146.4C191.15 146.4 207 130.55 207 111V35.4Z"
            fill="var(--calendar-ring)"
        />
        <path
            d="M439.8 35.4C439.8 15.85 423.95 0 404.4 0C384.85 0 369 15.85 369 35.4V111C369 130.55 384.85 146.4 404.4 146.4C423.95 146.4 439.8 130.55 439.8 111V35.4Z"
            fill="var(--calendar-ring)"
        />
        <rect x="91.8" y="242.4" width="100.8" height="96" rx="30" fill="var(--calendar-day)" />
        <rect x="240.6" y="242.4" width="100.8" height="96" rx="30" fill="var(--calendar-day)" />
        <rect x="389.4" y="242.4" width="100.8" height="96" rx="30" fill="var(--calendar-day)" />
        <rect x="91.8" y="367.2" width="100.8" height="96" rx="30" fill="var(--calendar-day)" />
        <rect x="240.6" y="367.2" width="100.8" height="96" rx="30" fill="var(--calendar-day)" />
        <rect x="389.4" y="367.2" width="100.8" height="96" rx="30" fill="var(--calendar-day)" />
    </svg>
);

export const IconFolder = (p: IconProps) => (
    <Svg {...p}>
        <path d="M4 4h5l2 3h9a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z" />
    </Svg>
);

export const IconTrash = ({ size = 20, className }: IconProps) => (
    <svg
        width={(size * 457) / 526}
        height={size}
        viewBox="0 0 457 526"
        fill="none"
        stroke="currentColor"
        strokeWidth={52}
        strokeLinecap="round"
        strokeLinejoin="round"
        className={className}
        aria-hidden="true"
        focusable="false"
    >
        <path d="M143 133V89C143 50 169 26 208 26H280C319 26 345 50 345 89V133" />
        <path d="M26 133H431" />
        <path d="M61 200V434C61 477 90 500 131 500H326C367 500 396 477 396 434V200" />
        <path d="M183 240V396M274 240V396" />
    </svg>
);

export const IconPlusCircle = ({ size = 20, className }: IconProps) => (
    <svg
        width={size}
        height={size}
        viewBox="0 0 516 516"
        fill="none"
        stroke="currentColor"
        strokeWidth={52}
        strokeLinecap="round"
        strokeLinejoin="round"
        className={className}
        aria-hidden="true"
        focusable="false"
    >
        <path d="M490 258C490 386.13 386.13 490 258 490C129.87 490 26 386.13 26 258C26 129.87 129.87 26 258 26C386.13 26 490 129.87 490 258Z" />
        <path d="M258 172V258M258 258V344M258 258H172M258 258H344" strokeWidth={60} />
    </svg>
);

/** Settings and security icons. */

export const IconSun = (p: IconProps) => (
    <Svg {...p}>
        <circle cx="12" cy="12" r="4" />
        <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
    </Svg>
);

export const IconMoon = (p: IconProps) => (
    <Svg {...p}>
        <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9z" />
    </Svg>
);

export const IconShield = (p: IconProps) => (
    <Svg {...p}>
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
        <path d="m9 11.5 2 2 4-4.5" />
    </Svg>
);

export const IconChart = (p: IconProps) => (
    <Svg {...p}>
        <path d="M3 3v18h18" />
        <path d="M8 17v-5M13 17V7M18 17v-8" />
    </Svg>
);

/** About page and social icons. */

export const IconGithub = ({ size = 20, className }: IconProps) => (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className} aria-hidden="true">
        <path
            fill="currentColor"
            d="M12 .5C5.65.5.5 5.65.5 12c0 5.08 3.29 9.39 7.86 10.91.58.11.79-.25.79-.56 0-.28-.01-1.02-.02-2-3.2.7-3.88-1.54-3.88-1.54-.52-1.33-1.28-1.68-1.28-1.68-1.04-.71.08-.7.08-.7 1.15.08 1.76 1.18 1.76 1.18 1.03 1.76 2.7 1.25 3.35.96.1-.75.4-1.25.72-1.54-2.55-.29-5.24-1.28-5.24-5.68 0-1.26.45-2.28 1.18-3.09-.12-.29-.51-1.46.11-3.05 0 0 .96-.31 3.15 1.18a10.9 10.9 0 0 1 5.74 0c2.19-1.49 3.15-1.18 3.15-1.18.62 1.59.23 2.76.11 3.05.73.81 1.18 1.83 1.18 3.09 0 4.41-2.69 5.38-5.25 5.67.41.35.77 1.05.77 2.12 0 1.53-.01 2.76-.01 3.14 0 .31.21.67.8.56A10.52 10.52 0 0 0 23.5 12C23.5 5.65 18.35.5 12 .5z"
        />
    </svg>
);

export const IconLink = (p: IconProps) => (
    <Svg {...p}>
        <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
        <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
    </Svg>
);

export const IconBug = (p: IconProps) => (
    <Svg {...p}>
        <path d="m8 2 1.88 1.88M14.12 3.88 16 2" />
        <path d="M9 7.13v-1a3 3 0 1 1 6 0v1" />
        <path d="M12 20c-3.3 0-6-2.7-6-6v-3a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v3c0 3.3-2.7 6-6 6z" />
        <path d="M12 20v-9" />
        <path d="M6 13H2M22 13h-4" />
        <path d="M6.5 9C4.6 8.8 3 7.1 3 5M17.5 9c1.9-.2 3.5-1.9 3.5-4" />
    </Svg>
);

export const IconHeart = (p: IconProps) => (
    <Svg {...p}>
        <path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.51 4.04 3 5.5l7 7z" />
    </Svg>
);

export const IconPackage = (p: IconProps) => (
    <Svg {...p}>
        <path d="M21 8.5 12 3 3 8.5v7L12 21l9-5.5v-7Z" />
        <path d="m3 8.5 9 5 9-5" />
        <path d="M12 13.5V21" />
    </Svg>
);

export const IconCopy = (p: IconProps) => (
    <Svg {...p}>
        <rect x="9" y="9" width="11" height="11" rx="2" />
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </Svg>
);

/** Bottom dock menu icons. */

export const IconMenuHome = ({ size = 20, className }: IconProps) => (
    <svg
        width={(size * 378) / 349}
        height={size}
        viewBox="-16 -16 378 349"
        fill="none"
        className={className}
        aria-hidden="true"
    >
        <path
            d="M39.72 271.39c0 24.38 19.75 43.98 43.98 43.98h176.04c24.23 0 43.97-19.6 43.97-43.98V164.45L172.69 59.45 39.72 164.45v106.94Z"
            fill="currentColor"
            opacity=".16"
        />
        <path
            d="M6.82 151.59c-11.07-14.51-8.38-35.15 5.98-46.22L131.26 14.13c24.38-18.85 58.48-18.85 83.01 0l118.31 91.24c14.51 11.07 17.2 31.71 5.98 46.22l.3-.3c-11.07 14.36-31.86 17.05-46.22 5.98L184.96 74.41c-7.18-5.53-17.2-5.53-24.38 0L52.89 157.27c-14.51 11.07-35.15 8.38-46.22-5.98l.15.3Z"
            fill="currentColor"
        />
        <path
            d="M133 219.01c0-17.65 14.36-32.01 32.01-32.01h13.01c17.65 0 32.01 14.36 32.01 32.01v62.97c0 8.38-6.73 15.11-15.11 15.11h-46.97c-8.23 0-14.96-6.73-14.96-15.11v-62.97Z"
            fill="currentColor"
        />
    </svg>
);

export const IconMenuDrivers = ({ size = 20, className }: IconProps) => (
    <svg
        width={(size * 377) / 315}
        height={size}
        viewBox="-12 -12 377 315"
        fill="none"
        className={className}
        aria-hidden="true"
    >
        <path
            d="M0 46.03C0 20.56 20.56 0 46.03 0h46.04c25.32 0 45.88 20.56 45.88 46.03v44.96c0 25.47-20.56 46.03-45.88 46.03H46.03C20.56 137.03 0 116.47 0 90.99V46.03Zm38.98 8.9c0-8.75 7.21-15.96 15.96-15.96h27c8.9 0 16.11 7.21 16.11 15.96v27.01c0 8.9-7.21 16.11-16.11 16.11h-27c-8.75 0-15.96-7.21-15.96-16.11V54.93Z"
            fill="currentColor"
            fillRule="evenodd"
            clipRule="evenodd"
        />
        <path
            d="M1.07 207c0-25.47 20.56-46.03 45.88-46.03h47.11c25.32 0 45.88 20.56 45.88 46.03v36.98c0 25.47-20.56 46.03-45.88 46.03H46.96c-25.32 0-45.88-20.56-45.88-46.03V207Z"
            fill="currentColor"
        />
        <path
            d="M164.04 50.02c0-16.57 13.35-30.07 29.92-30.07h127.98c16.58 0 30.08 13.5 30.08 30.07 0 16.58-13.5 29.93-30.08 29.93H193.96c-16.57 0-29.92-13.35-29.92-29.93Z"
            fill="currentColor"
            opacity=".45"
        />
        <path
            d="M164.04 157.44c0-16.27 13.19-29.47 29.46-29.47h129.05c16.27 0 29.47 13.19 29.47 29.47 0 16.42-13.19 29.61-29.47 29.61H193.5c-16.27 0-29.46-13.19-29.46-29.61Z"
            fill="currentColor"
            opacity=".45"
        />
        <path
            d="M164.04 253.5c0-16.27 13.19-29.46 29.46-29.46h129.05c16.27 0 29.47 13.19 29.47 29.46 0 16.27-13.19 29.47-29.47 29.47H193.5c-16.27 0-29.46-13.19-29.46-29.47Z"
            fill="currentColor"
            opacity=".45"
        />
    </svg>
);

export const IconMenuComponents = ({ size = 20, className }: IconProps) => (
    <svg
        width={(size * 384) / 385}
        height={size}
        viewBox="-12 -12 384 385"
        fill="none"
        className={className}
        aria-hidden="true"
    >
        <path
            d="M163.51 177.61c10.3-4.53 22.16-4.53 32.46 0l147.92 65.37c19.66 8.74 19.82 36.67.31 45.56l-147.76 67.41c-10.61 4.84-22.78 4.84-33.39 0L15.28 288.55c-19.5-8.89-19.35-36.82.31-45.56l147.92-65.38Z"
            fill="currentColor"
            opacity=".3"
        />
        <path
            d="M162.58 93.5c10.92-5.15 23.4-5.15 34.32 0l147.45 69.75c19.04 8.89 19.19 35.73.31 45.09l-147.3 72.09c-11.08 5.46-24.18 5.46-35.26 0L14.81 208.34c-18.88-9.36-18.72-36.2.31-45.09L162.58 93.5Z"
            fill="currentColor"
            opacity=".55"
        />
        <path
            d="M162.11 4.1c11.08-5.46 24.18-5.46 35.26 0l143.55 70.84c23.72 11.7 23.72 45.4.16 57.26l-143.55 71.62c-11.23 5.62-24.34 5.62-35.58 0L18.4 132.2C-5.16 120.34-5.16 86.64 18.56 74.94L162.11 4.1Z"
            fill="currentColor"
        />
    </svg>
);

export const IconMenuBackups = ({ size = 20, className }: IconProps) => (
    <svg
        width={(size * 416) / 354}
        height={size}
        viewBox="-12 -12 416 354"
        fill="none"
        className={className}
        aria-hidden="true"
    >
        <path
            d="M.75 222.14c0-17.66 14.27-32.1 31.93-32.1h1.02c17.66 0 32.1 14.44 32.1 32.1v28.87c0 10.02 7.98 18 18 18h224.86c10.02 0 18-7.98 18-18v-28.87c0-17.66 14.43-32.1 32.1-32.1 17.66 0 31.93 14.44 31.93 32.1v46.87c0 33.12-26.83 60.12-59.95 60.12H60.7c-33.12 0-59.95-27-59.95-60.12v-46.87Z"
            fill="currentColor"
            opacity=".35"
        />
        <path
            d="M155.63 41.1C155.63 18.34 174.14 0 196.73 0h1.02c22.59 0 40.93 18.34 40.93 41.1v88.65c0 2.72 3.23 4.08 5.1 2.21l12.4-12.4c11.38-11.38 29.72-11.38 41.1 0 11.2 11.38 11.2 29.72 0 41.1l-81.69 81.52c-10.19 10.19-26.66 10.19-36.68 0l-81.69-81.52c-11.38-11.38-11.38-29.72 0-41.1 11.38-11.38 29.72-11.38 40.93 0l12.4 12.4c1.87 1.87 5.1.51 5.1-2.21V41.1Z"
            fill="currentColor"
        />
    </svg>
);

export const IconMenuSettings = ({ size = 20, className }: IconProps) => (
    <svg
        width={(size * 387) / 392}
        height={size}
        viewBox="-24 -24 387 392"
        fill="none"
        className={className}
        aria-hidden="true"
    >
        <path
            d="M186.19 0c23 0 39 16 44 38l3 13 16 8 13-7c20-11 43-3 54 16l17 29c11 20 4 43-16 54l-12 7v19l12 7c20 11 27 34 16 54l-17 29c-11 19-34 27-54 16l-13-7-16 8-3 13c-5 22-21 38-44 38h-34c-23 0-39-16-44-38l-3-13-16-8-13 7c-20 11-43 3-54-16l-17-29c-11-20-4-43 16-54l12-7v-19l-12-7c-20-11-27-34-16-54l17-29c11-19 34-27 54-16l13 7 16-8 3-13c5-22 21-38 44-38h34Z"
            fill="currentColor"
        />
        <circle cx="169.19" cy="168" r="72" fill="var(--c-card)" />
        <circle cx="169.19" cy="168" r="37" fill="currentColor" opacity=".35" />
    </svg>
);

/** Settings row icons. */

/**
 * Renders a settings icon with the shared 1.8 px stroke.
 *
 * @param props icon properties
 * @returns SVG at the row icon size
 */
function SettingsLineIcon({ children, size = 21, className }: IconProps & { children: ReactNode }) {
    return (
        <Svg size={size} className={className} strokeWidth={1.8}>
            {children}
        </Svg>
    );
}

export const IconLineGlobe = (p: IconProps) => (
    <SettingsLineIcon {...p}>
        <circle cx="12" cy="12" r="9" />
        <path d="M3 12h18M12 3a14.5 14.5 0 0 1 0 18M12 3a14.5 14.5 0 0 0 0 18M5 8.5h14M5 15.5h14" />
    </SettingsLineIcon>
);

export const IconLinePalette = (p: IconProps) => (
    <SettingsLineIcon {...p}>
        <path d="M12 3a9 9 0 1 0 0 18c1.7 0 2.5-1 2.5-2.2 0-1.5-1.5-1.8-1.5-3 0-1.1.9-1.8 2-1.8h1.5A4.5 4.5 0 0 0 21 9.5C21 5.9 17 3 12 3Z" />
        <circle cx="7.5" cy="9" r="1" fill="currentColor" stroke="none" />
        <circle cx="10.5" cy="6.8" r="1" fill="currentColor" stroke="none" />
        <circle cx="14.5" cy="6.8" r="1" fill="currentColor" stroke="none" />
        <circle cx="6.5" cy="13" r="1" fill="currentColor" stroke="none" />
    </SettingsLineIcon>
);

export const IconLineBell = (p: IconProps) => (
    <SettingsLineIcon {...p}>
        <path d="M18 9a6 6 0 0 0-12 0c0 5-2 6-2 6h16s-2-1-2-6" />
        <path d="M10 19a2 2 0 0 0 4 0" />
    </SettingsLineIcon>
);

export const IconLineSync = (p: IconProps) => (
    <SettingsLineIcon {...p}>
        <path d="M20 12a8 8 0 0 1-13.6 5.7L4 15" />
        <path d="M4 12a8 8 0 0 1 13.6-5.7L20 9" />
        <path d="M20 4v5h-5M4 20v-5h5" />
    </SettingsLineIcon>
);

export const IconLineClock = (p: IconProps) => (
    <SettingsLineIcon {...p}>
        <circle cx="12" cy="12" r="9" />
        <path d="M12 7v5l3.5 2" />
    </SettingsLineIcon>
);

export const IconLineShield = (p: IconProps) => (
    <SettingsLineIcon {...p}>
        <path d="M12 3 4.5 6v5.5c0 4.4 3 8.2 7.5 9.5 4.5-1.3 7.5-5.1 7.5-9.5V6L12 3Z" />
        <path d="m8.5 12 2.3 2.3 4.7-4.6" />
    </SettingsLineIcon>
);

export const IconLineDatabase = (p: IconProps) => (
    <SettingsLineIcon {...p}>
        <ellipse cx="12" cy="6" rx="7.5" ry="3" />
        <path d="M4.5 6v6c0 1.7 3.4 3 7.5 3s7.5-1.3 7.5-3V6" />
        <path d="M4.5 12v6c0 1.7 3.4 3 7.5 3s7.5-1.3 7.5-3v-6" />
    </SettingsLineIcon>
);

export const IconLineFolder = (p: IconProps) => (
    <SettingsLineIcon {...p}>
        <path d="M3.5 7.5A2.5 2.5 0 0 1 6 5h4l2 2h6a2.5 2.5 0 0 1 2.5 2.5v7A2.5 2.5 0 0 1 18 19H6a2.5 2.5 0 0 1-2.5-2.5v-9Z" />
    </SettingsLineIcon>
);

export const IconLineChart = (p: IconProps) => (
    <SettingsLineIcon {...p}>
        <path d="M4 20V10M10 20V4M16 20v-7M22 20H2" />
    </SettingsLineIcon>
);

export const IconLineCards = (p: IconProps) => (
    <SettingsLineIcon {...p}>
        <rect x="8" y="8" width="12" height="12" rx="2" />
        <path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2" />
    </SettingsLineIcon>
);

/** Theme segmented control icons. */

export const IconWindows = ({ size = 20, className }: IconProps) => (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className} aria-hidden="true">
        <path
            fill="currentColor"
            d="M3 5.4 10.6 4.3v7.2H3zM10.6 12.5v7.2L3 18.6v-6.1zM11.6 4.2 21 3v8.5h-9.4zM21 12.5V21l-9.4-1.2v-7.3z"
        />
    </svg>
);

/** Device glyphs used by driver rows. */

const IconGpu = (p: IconProps) => (
    <Svg {...p}>
        <rect x="2.5" y="6.5" width="19" height="11" rx="2" />
        <circle cx="9.5" cy="12" r="3" />
        <path d="M16 10h3M16 14h3" />
        <path d="M6 17.5V20M18 17.5V20" />
    </Svg>
);

const IconAudio = (p: IconProps) => (
    <Svg {...p}>
        <path d="M11 5 6 9H2v6h4l5 4V5z" />
        <path d="M15.5 8.5a5 5 0 0 1 0 7" />
        <path d="M18.4 5.6a9 9 0 0 1 0 12.8" />
    </Svg>
);

const IconChip = (p: IconProps) => (
    <Svg {...p}>
        <rect x="7" y="7" width="10" height="10" rx="2" />
        <rect x="10.2" y="10.2" width="3.6" height="3.6" rx="0.8" />
        <path d="M10 2v3M14 2v3M10 19v3M14 19v3M2 10h3M2 14h3M19 10h3M19 14h3" />
    </Svg>
);

const IconNetwork = (p: IconProps) => (
    <Svg {...p}>
        <rect x="3" y="9" width="18" height="10" rx="2" />
        <path d="M7.5 13h2M11 13h2M14.5 13h2" />
        <path d="M12 9V5M8 5h8" />
    </Svg>
);

const IconDisplay = (p: IconProps) => (
    <Svg {...p}>
        <rect x="2" y="3" width="20" height="14" rx="2" />
        <path d="M8 21h8M12 17v4" />
        <path d="m6 13 4-5 3 3.5 2-2 3 3.5" />
    </Svg>
);

const IconInput = (p: IconProps) => (
    <Svg {...p}>
        <path d="M17.32 6H6.68a4 4 0 0 0-3.98 3.6c-.01.07-.7 6.9-.7 7.4a3 3 0 0 0 3 3c1 0 1.5-.5 2-1l1.5-1.5c.5-.5 1-.5 1.5-.5h4c.5 0 1 0 1.5.5L17 19c.5.5 1 1 2 1a3 3 0 0 0 3-3c0-.5-.69-7.33-.7-7.4A4 4 0 0 0 17.32 6z" />
        <path d="M6.5 12h3M8 10.5v3" />
        <path d="M15.5 11.5h.01M17.5 13.5h.01" />
    </Svg>
);

const GLYPHS: Record<DriverIconKind, (p: IconProps) => ReactNode> = {
    gpu: IconGpu,
    audio: IconAudio,
    chip: IconChip,
    network: IconNetwork,
    display: IconDisplay,
    input: IconInput,
};

export function DriverGlyph({ kind, color, size = 26 }: { kind: DriverIconKind; color: string; size?: number }) {
    const Glyph = GLYPHS[kind] ?? IconChip;
    return (
        <span className="driver-glyph" style={{ color }}>
            <Glyph size={size} strokeWidth={1.7} />
        </span>
    );
}
