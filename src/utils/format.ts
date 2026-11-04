import type { TranslateParams } from '../i18n';

/** `en` keeps the previous en-GB output; every other locale uses its own tag. */
function dateLocale(locale: string): string {
    return locale === 'en' ? 'en-GB' : locale;
}

function formatWith(locale: string, options: Intl.DateTimeFormatOptions, date: Date): string {
    try {
        return new Intl.DateTimeFormat(dateLocale(locale), options).format(date);
    } catch {
        return new Intl.DateTimeFormat('en-GB', options).format(date);
    }
}

export function formatDateTime(iso: string, locale: string): string {
    const date = new Date(iso);
    if (Number.isNaN(date.getTime())) return iso;
    return formatWith(
        locale,
        {
            day: '2-digit',
            month: '2-digit',
            year: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
        },
        date,
    );
}

export function formatDate(iso: string, locale: string): string {
    const date = new Date(iso);
    if (Number.isNaN(date.getTime())) return iso;
    return formatWith(locale, { day: '2-digit', month: '2-digit', year: 'numeric' }, date);
}

function sameDay(a: Date, b: Date): boolean {
    return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

/** "Сегодня, 21:47" / "Yesterday, 21:47" / "02.11.2025, 14:32" */
export function formatRelativeDay(
    iso: string | null,
    locale: string,
    t: (key: string, params?: TranslateParams & { count?: number }) => string,
): string {
    if (!iso) return t('common.never');
    const date = new Date(iso);
    if (Number.isNaN(date.getTime())) return iso;
    const now = new Date();
    const time = formatWith(locale, { hour: '2-digit', minute: '2-digit' }, date);
    if (sameDay(date, now)) return `${t('common.today')}, ${time}`;
    const yesterday = new Date(now);
    yesterday.setDate(now.getDate() - 1);
    if (sameDay(date, yesterday)) return `${t('common.yesterday')}, ${time}`;
    return formatDateTime(iso, locale);
}
