# Driver Updater

**Бесплатный менеджер драйверов с открытым исходным кодом.**
Десктопное приложение на [Tauri 2](https://v2.tauri.app/) + React + TypeScript: просмотр
установленных драйверов, поиск и установка обновлений, точки восстановления и резервные
копии — в современном интерфейсе, отрисованном по макетам дизайн-команды.

> ⚠️ Текущий этап: **интерфейс + тестовые данные**. Реальное определение драйверов
> подключается позже через слой `Backend`/`MockRepository` (см. «Архитектура»).
> Вся бизнес-логика UI уже работает поверх IPC-контракта, поэтому замена mock-слоя на
> реальную детекцию не затрагивает фронтенд.

---

## Возможности (уже в UI)

- **Главная** — состояние системы, сводка (устройства / актуальные / обновления), последняя проверка, рекомендуемые обновления.
- **Драйверы** — поиск, фильтр по категориям, режимы «Список» и «Категории», карточки драйверов с версиями, статусами и подробностями, установка обновлений по одному.
- **Обновления** — выбор обновлений чекбоксами, суммарный размер, пакетная установка, разделение «Рекомендуется» / «Необязательно».
- **Резервные копии** — список точек восстановления, создание/удаление/откат (с подтверждениями), автоматическое создание по расписанию.
- **Настройки** — язык интерфейса, тема (светлая/тёмная/системная), уведомления, интервалы проверок, безопасность, путь для бэкапов, телеметрия.
- **О проекте** — версия, лицензия, ссылки на репозиторий, системная информация.
- **i18n**: русский и английский из коробки + механизм подключаемых локалей от сообщества.
- Кастомный titlebar (frameless-окно Tauri) с drag-region и кнопками управления окном.

## Быстрый старт

Требуется: Node.js 20+, Rust stable (для десктоп-режима), для Linux — зависимости Tauri
(`libwebkit2gtk-4.1-dev`, `libgtk-3-dev` и т. д.).

```bash
npm install

# Веб-режим (интерфейс с mock-бэкендом, без Rust) — удобен для разработки UI
npm run dev

# Десктоп-режим (Tauri + Rust-ядро с mock-репозиторием)
npm run tauri dev

# Проверки
npm run typecheck
npm test
npm run build
```

Веб-режим автоматически использует `MockBackend` (те же фикстуры), десктоп — IPC-вызовы
Rust-ядра: выбор реализации происходит в `src/ipc/backend.ts` по наличию
`window.__TAURI_INTERNALS__`.

## Структура проекта

```
├── shared/fixtures/        # ЕДИНЫЙ источник тестовых данных (читают и TS, и Rust)
│   ├── drivers.json        #   драйверы: версии, статусы, размеры, описания ru/en
│   ├── restore_points.json #   точки восстановления
│   ├── settings.json       #   настройки по умолчанию
│   └── meta.json           #   сводные счётчики устройств
├── src/                    # Фронтенд (React + TS + Vite)
│   ├── ipc/                #   контракт Backend: tauri.ts (IPC) и mock.ts (браузер)
│   ├── i18n/               #   словари ru/en, plural-формы, registerLocale()
│   ├── store/app.tsx       #   состояние приложения и действия поверх Backend
│   ├── components/         #   UI-кит, иконки (inline SVG), иллюстрации, titlebar, sidebar
│   ├── pages/              #   шесть страниц приложения
│   └── styles/             #   токены тем (светлая/тёмная) + компоненты + страницы
├── src-tauri/              # Ядро (Rust)
│   ├── src/commands.rs     #   IPC-команды (async, с имитацией задержек)
│   ├── src/mock.rs         #   MockRepository: фикстуры + персист в app data dir
│   ├── src/models.rs       #   serde-модели, зеркалят src/ipc/types.ts
│   └── src/lib.rs          #   сборка приложения, состояние, invoke_handler
└── scripts/generate_icons.py  # генерация иконок без внешних зависимостей
```

## Архитектура и IPC-контракт

Фронтенд ничего не знает о происхождении данных: он говорит с интерфейсом `Backend`
(`src/ipc/backend.ts`). Реализации:

| Команда (Rust)          | Метод Backend          | Назначение                                  |
| ----------------------- | ---------------------- | ------------------------------------------- |
| `get_initial_data`      | `getInitialData()`     | снапшот: драйверы, точки, настройки, сводка |
| `check_updates`         | `checkUpdates()`       | сканирование (обновляет `lastCheck`)        |
| `install_driver`        | `installDriver(id)`    | установка одного обновления                 |
| `install_updates`       | `installUpdates(ids)`  | пакетная установка                          |
| `list_restore_points`   | `listRestorePoints()`  | список точек восстановления                 |
| `create_restore_point`  | `createRestorePoint()` | создать точку                               |
| `delete_restore_point`  | `deleteRestorePoint()` | удалить точку                               |
| `restore_from_point`    | `restoreFromPoint()`   | откат к точке                               |
| `save_settings`         | `saveSettings(s)`      | сохранить настройки                         |
| `get_system_info`       | `getSystemInfo()`      | ОС / архитектура / платформа                |
| `compute_summary`       | `computeSummary()`     | пересчёт сводки после изменений             |

Когда появится реальное определение драйверов, достаточно заменить `MockRepository`
в `src-tauri/src/mock.rs` на боевую реализацию того же набора методов — контракты
и фронтенд не меняются.

## i18n и подключаемые локали

Словари — типизированные TS-модули (`src/i18n/ru.ts` — эталон, `en.ts` — зеркало).
Плюральные формы выбираются через `Intl.PluralRules`. Сообщество может добавлять
локали без правки ядра:

```ts
import { registerLocale } from './i18n';

registerLocale('tt', 'Татарча', dictionary as Dictionary);
```

Недостающие ключи любой дополнительной локали падают обратно на английский.

## Тестирование

- `npm test` — unit-тесты (сравнение версий, i18n/plurals/фолбэки).
- `npm run typecheck` — строгий TypeScript (`strict`, `noUnusedLocals`).
- CI (`.github/workflows/ci.yml`) прогоняет фронтенд и `cargo check` ядра.

## Вклад в проект

См. [CONTRIBUTING.md](CONTRIBUTING.md). Коротко: форк → ветка → PR; коммиты в стиле
Conventional Commits; новые строки UI — сразу в оба словаря; фикстуры меняются только
в `shared/fixtures/`.

## Лицензия

MIT © Driver Updater Community. См. [LICENSE](LICENSE).

---

## English (summary)

Driver Updater is a free, open-source driver manager built with Tauri 2 + React + TS.
The current milestone ships the full interface on top of test data: real hardware
detection plugs in later behind the `Backend` / `MockRepository` boundary, so the
frontend contract stays untouched. `npm run dev` starts a web preview with the mock
backend; `npm run tauri dev` runs the desktop shell. Fixtures in `shared/fixtures/`
are the single source of test data for both halves of the app. MIT licensed.
