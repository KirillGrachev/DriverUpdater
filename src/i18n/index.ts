import en from './locales/en.json';
import ru from './locales/ru.json';
import type { Dictionary } from './types';

export type LocaleCode = string;

type DictValue = string | { [plural: string]: string };
type DictTree = { [key: string]: DictValue | DictTree };

/**
 * Locale bundles are separate JSON chunks, loaded on demand.
 */
const localeModules = import.meta.glob<{ default: Dictionary }>('./locales/*.json');

const baseLocales: Record<string, Dictionary> = { ru, en };
const loadedLocales = new Map<string, Dictionary>();

/**
 * Community/extension locales registered at runtime (see registerLocale).
 */
const extraLocales = new Map<string, { label: string; dict: Dictionary }>();

/**
 * Same order as the BirthdayBot language picker.
 */
const LOCALE_ORDER = [
    'ru',
    'uk',
    'be',
    'en',
    'de',
    'es',
    'fr',
    'it',
    'pt',
    'pl',
    'cs',
    'sk',
    'bg',
    'hr',
    'sl',
    'nl',
    'da',
    'sv',
    'fi',
    'et',
    'lv',
    'lt',
    'ro',
    'hu',
    'el',
    'ga',
    'mt',
    'kk',
    'ky',
    'tg',
    'tk',
    'uz',
    'az',
    'ka',
    'hy',
];

export const localeLabels: Record<string, string> = {
    ru: 'Русский',
    uk: 'Українська',
    be: 'Беларуская',
    en: 'English',
    de: 'Deutsch',
    es: 'Español',
    fr: 'Français',
    it: 'Italiano',
    pt: 'Português',
    pl: 'Polski',
    cs: 'Čeština',
    sk: 'Slovenčina',
    bg: 'Български',
    hr: 'Hrvatski',
    sl: 'Slovenščina',
    nl: 'Nederlands',
    da: 'Dansk',
    sv: 'Svenska',
    fi: 'Suomi',
    et: 'Eesti',
    lv: 'Latviešu',
    lt: 'Lietuvių',
    ro: 'Română',
    hu: 'Magyar',
    el: 'Ελληνικά',
    ga: 'Gaeilge',
    mt: 'Malti',
    kk: 'Қазақша',
    ky: 'Кыргызча',
    tg: 'Тоҷикӣ',
    tk: 'Türkmençe',
    uz: 'Oʻzbekcha',
    az: 'Azərbaycanca',
    ka: 'ქართული',
    hy: 'Հայերեն',
};

export function registerLocale(code: string, label: string, dict: Dictionary): void {
    extraLocales.set(code, { label, dict });
    localeLabels[code] = label;
}

/**
 * Load one JSON bundle once; returns false when the locale has no bundle.
 */
export async function loadLocale(code: string): Promise<boolean> {
    if (code === 'ru' || code === 'en') return true;
    if (loadedLocales.has(code) || extraLocales.has(code)) return true;
    const loader = localeModules[`./locales/${code}.json`];
    if (!loader) return false;
    try {
        const module = await loader();
        loadedLocales.set(code, module.default);
        return true;
    } catch {
        return false;
    }
}

export function availableLocales(): { code: LocaleCode; label: string }[] {
    const rest = [...extraLocales.keys()].filter((code) => !LOCALE_ORDER.includes(code));
    return [...LOCALE_ORDER, ...rest].map((code) => ({
        code,
        label: localeLabels[code] ?? code,
    }));
}

export function isSupportedLocale(code: string): boolean {
    return code === 'ru' || code === 'en' || LOCALE_ORDER.includes(code) || extraLocales.has(code);
}

export function getDictionary(code: LocaleCode): Dictionary {
    return extraLocales.get(code)?.dict ?? loadedLocales.get(code) ?? baseLocales[code] ?? baseLocales.en;
}

function lookup(dict: DictTree, path: string): DictValue | undefined {
    let node: DictValue | DictTree | undefined = dict;
    for (const part of path.split('.')) {
        if (typeof node !== 'object' || node === null) return undefined;
        node = (node as DictTree)[part];
    }
    return typeof node === 'string' || (typeof node === 'object' && node !== null) ? (node as DictValue) : undefined;
}

const pluralRules = new Map<string, Intl.PluralRules>();

function pluralCategory(locale: LocaleCode, n: number): string {
    let rules = pluralRules.get(locale);
    if (!rules) {
        try {
            rules = new Intl.PluralRules(locale);
        } catch {
            return 'other';
        }
        pluralRules.set(locale, rules);
    }
    return rules.select(n);
}

export interface TranslateParams {
    [key: string]: string | number;
}

/**
 * Resolve a dotted key, apply plural forms and interpolate {params}.
 */
export function translate(locale: LocaleCode, key: string, params?: TranslateParams & { count?: number }): string {
    const dict = getDictionary(locale) as unknown as DictTree;
    const fallback = getDictionary('en') as unknown as DictTree;
    const raw = lookup(dict, key) ?? lookup(fallback, key);
    if (raw === undefined) return key;

    let text: string;
    if (typeof raw === 'string') {
        text = raw;
    } else {
        const count = params?.count;
        const category = count === undefined ? 'other' : pluralCategory(locale, count);
        text = raw[category] ?? raw.other ?? raw.many ?? Object.values(raw)[0] ?? key;
    }

    if (params) {
        for (const [name, value] of Object.entries(params)) {
            if (name === 'count') continue;
            text = text.replaceAll(`{${name}}`, String(value));
        }
        if ('count' in params) text = text.replaceAll('{n}', String(params.count));
    }
    return text;
}
