import ru, { type Dictionary } from './ru';
import en from './en';

export type LocaleCode = string;

type DictValue = string | { [plural: string]: string };
type DictTree = { [key: string]: DictValue | DictTree };

const baseLocales: Record<string, Dictionary> = { ru, en };

/** Community/extension locales registered at runtime (see registerLocale). */
const extraLocales = new Map<string, { label: string; dict: Dictionary }>();

export const localeLabels: Record<string, string> = {
  ru: 'Русский',
  en: 'English',
};

export function registerLocale(code: string, label: string, dict: Dictionary): void {
  extraLocales.set(code, { label, dict });
  localeLabels[code] = label;
}

export function availableLocales(): { code: LocaleCode; label: string }[] {
  return [...Object.keys(baseLocales), ...extraLocales.keys()].map((code) => ({
    code,
    label: localeLabels[code] ?? code,
  }));
}

export function getDictionary(code: LocaleCode): Dictionary {
  return extraLocales.get(code)?.dict ?? baseLocales[code] ?? baseLocales.en;
}

function lookup(dict: DictTree, path: string): DictValue | undefined {
  let node: DictValue | DictTree | undefined = dict;
  for (const part of path.split('.')) {
    if (typeof node !== 'object' || node === null) return undefined;
    node = (node as DictTree)[part];
  }
  return typeof node === 'string' || (typeof node === 'object' && node !== null)
    ? (node as DictValue)
    : undefined;
}

const pluralRules = new Map<string, Intl.PluralRules>();

function pluralCategory(locale: LocaleCode, n: number): string {
  let rules = pluralRules.get(locale);
  if (!rules) {
    rules = new Intl.PluralRules(locale);
    pluralRules.set(locale, rules);
  }
  return rules.select(n);
}

export interface TranslateParams {
  [key: string]: string | number;
}

/** Resolve a dotted key, apply plural forms and interpolate {params}. */
export function translate(
  locale: LocaleCode,
  key: string,
  params?: TranslateParams & { count?: number },
): string {
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
