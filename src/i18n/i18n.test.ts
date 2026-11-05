import { describe, expect, it } from 'vitest';
import type { Dictionary } from './types';
import { availableLocales, registerLocale, translate } from './index';

describe('i18n', () => {
    it('resolves plain keys per locale', () => {
        expect(translate('ru', 'nav.home')).toBe('Главная');
        expect(translate('en', 'nav.home')).toBe('Home');
    });

    it('interpolates params', () => {
        expect(translate('ru', 'updates.selected', { n: 3 })).toBe('Выбрано обновлений: 3');
        expect(translate('en', 'updates.selected', { n: 5 })).toBe('Selected updates: 5');
    });

    it('applies russian plural rules', () => {
        expect(translate('ru', 'toasts.scanFound', { count: 1 })).toBe('Найдено 1 обновление');
        expect(translate('ru', 'toasts.scanFound', { count: 3 })).toBe('Найдено 3 обновления');
        expect(translate('ru', 'toasts.scanFound', { count: 5 })).toBe('Найдено 5 обновлений');
        expect(translate('en', 'toasts.scanFound', { count: 1 })).toBe('1 update found');
        expect(translate('en', 'toasts.scanFound', { count: 4 })).toBe('4 updates found');
    });

    it('falls back to english and then to the key', () => {
        expect(translate('ru', 'nav.home')).toBe('Главная');
        expect(translate('xx', 'nav.home')).toBe('Home');
        expect(translate('ru', 'does.not.exist')).toBe('does.not.exist');
    });

    it('supports community locales via registerLocale', () => {
        const partial = { nav: { home: 'Баш бит' } } as unknown as Dictionary;
        registerLocale('tt', 'Татарча', partial);
        expect(availableLocales().some((l) => l.code === 'tt')).toBe(true);
        expect(translate('tt', 'nav.home')).toBe('Баш бит');
        /**
         * missing keys fall back to english
         */
        expect(translate('tt', 'nav.drivers')).toBe('Drivers');
    });
});
