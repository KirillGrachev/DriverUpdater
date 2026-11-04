# Вклад в Driver Updater / Contributing

Спасибо, что хотите помочь проекту! / Thanks for wanting to help the project!

## Правила / Ground rules

- Коммиты — [Conventional Commits](https://www.conventionalcommits.org/):
  `feat:`, `fix:`, `chore:`, `docs:`, `test:`, `refactor:`.
- Ветки: `feat/...`, `fix/...`, `chore/...` от `main`.
- PR должен проходить `npm run typecheck`, `npm test`, `npm run build`.
- Новые строки интерфейса добавляются **сразу в оба словаря** (`src/i18n/ru.ts`,
  `src/i18n/en.ts`); плюральные формы — объектом `{ one, few, many, other }` /
  `{ one, other }`.
- Тестовые данные меняются только в `shared/fixtures/*.json` — их читают и фронтенд,
  и Rust-ядро.
- Иконки и иллюстрации — только inline SVG (приложение работает офлайн).

## Локальный цикл / Local loop

```bash
npm install
npm run dev        # веб-режим с mock-бэкендом
npm run tauri dev  # десктоп-режим (нужен Rust + зависимости Tauri)
npm test
```

## Подключаемые локали / Community locales

Добавьте словарь и зарегистрируйте его через `registerLocale(code, label, dict)`
(`src/i18n/index.ts`); недостающие ключи автоматически падают на английский.
PR с новой локалью приветствуется отдельным коммитом `feat(i18n): add <code> locale`.

## Сообщение об ошибках / Reporting bugs

Создайте issue в репозитории с шагом воспроизведения, ОС и версией приложения
(экран «О проекте»).
