/**
 * Plural forms selected through Intl.PluralRules.
 */
export type PluralForms = { one?: string; few?: string; many?: string; other: string };
export type DictValue = string | PluralForms;
export type DictNode = { [key: string]: DictValue | DictNode };
export type Dictionary = DictNode;
