# Kley Publish Hooks — Spec v1

## Контекст и мотивация

`kley publish` по умолчанию — чистое копирование файлов пакета в локальный store, без исполнения каких-либо `package.json` lifecycle-скриптов. Это осознанное архитектурное отличие от `yalc`, где implicit-запуск npm-хуков (`prepublish`, `prepack`, `publish`, `postpublish` и т.д.) приводит к классу багов, когда локальный инструмент случайно триггерит реальную публикацию в npm registry (см. [yalc#255](https://github.com/wclr/yalc/issues/255)).

Проблема: многие пакеты (особенно TypeScript) хранят шаг сборки именно в `prepublishOnly`/`prepack`/`prepare`. Полный отказ от их выполнения перекладывает на пользователя ручную сборку перед каждым `kley publish`.

Решение: явный, управляемый пользователем набор хуков, хранящийся отдельно от `package.json` — в `.kley/hooks.json`. Kley **никогда** не читает `scripts` из `package.json` в рантайме публикации — только из этого файла. Файл заполняется через интерактивный визард при первой публикации.

---

## 1. Формат `.kley/hooks.json`

Плоский объект: ключ — имя npm-хука, значение — команда.

```jsonc
// .kley/hooks.json (в корне пакета, рядом с package.json)
{
  "prepare":        { "command": "npm run build" },
  "prepack":        { "command": "tsc -p tsconfig.json" },
  "prepublishOnly": { "command": "npm test" }
}
```

В файле присутствуют **только** те хуки, которые пользователь явно выбрал в визарде. Хуки, от которых пользователь отказался (не отметил), в файл не попадают вовсе — никаких записей с `enabled: false`.

Фаза (`pre`/`post`) не хранится в файле — выводится из статической таблицы имён хуков, зашитой в kley:

```rust
// src/hooks/registry.rs

pub enum HookPhase {
    Pre,
    Post,
}

pub const KNOWN_HOOKS: &[(&str, HookPhase)] = &[
    ("prepare",        HookPhase::Pre),
    ("prepack",        HookPhase::Pre),
    ("prepublishOnly", HookPhase::Pre),
    ("postpack",       HookPhase::Post),
    ("publish",        HookPhase::Post),
    ("postpublish",    HookPhase::Post),
];
```

Порядок выполнения внутри фазы — фиксированный, по порядку объявления в `KNOWN_HOOKS` (совпадает с реальным npm lifecycle order), а не порядок ключей в JSON.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HookEntry {
    pub command: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HooksConfig {
    #[serde(flatten)]
    pub hooks: HashMap<String, HookEntry>,
}
```

---

## 2. Flow команды `kley publish`

Ключевое правило: **сам факт существования `.kley/hooks.json` означает, что визард уже отработал** — даже если файл пустой (`{}`). Это единственный сигнал "спрашивать повторно не нужно".

```rust
pub fn run_publish(ctx: &PublishContext) -> Result<()> {
    let hooks_path = ctx.repo_root.join(".kley/hooks.json");

    let hooks_config = if hooks_path.exists() {
        // визард уже отработал ранее (файл мог быть и пустым `{}`) — просто читаем
        HooksConfig::load(&hooks_path)?
    } else if !ctx.non_interactive {
        // файла нет + обычный режим → запускаем визард
        run_hooks_wizard(ctx, &hooks_path)?
    } else {
        // файла нет + --non-interactive → ничего не делаем, чистое копирование
        HooksConfig::default()
    };

    run_phase(&hooks_config, HookPhase::Pre, ctx)?;
    copy_package_files(ctx)?;      // текущее поведение publish, без изменений
    run_phase(&hooks_config, HookPhase::Post, ctx)?;

    Ok(())
}
```

### Non-interactive режим

Управляется явным флагом, а не автоопределением TTY:

```bash
kley publish --non-interactive
```

Поведение:
- Если `.kley/hooks.json` существует → хуки читаются из него и выполняются как обычно.
- Если файла нет → визард **не запускается**, публикация идёт как чистое копирование, без хуков.

Это значит: чтобы CI выполнял хуки, `.kley/hooks.json` должен быть создан заранее (локально, через обычный интерактивный `kley publish`) и присутствовать в файловой системе на момент CI-запуска — каким способом он туда попадёт (git, артефакт сборки, ручное копирование) остаётся на усмотрение пользователя и вне рамок этой спецификации (см. п.5 ниже).

---

## 3. Визард (batch-режим)

Один экран со всеми найденными хуками сразу, чекбоксы, без записи в файл ничего лишнего.

```rust
fn run_hooks_wizard(ctx: &PublishContext, hooks_path: &Path) -> Result<HooksConfig> {
    let pkg_scripts = read_package_json_scripts(ctx)?;   // HashMap<String, String>

    let candidates: Vec<(&str, &str)> = KNOWN_HOOKS.iter()
        .filter_map(|(name, _)| pkg_scripts.get(*name).map(|cmd| (*name, cmd.as_str())))
        .collect();

    let mut config = HooksConfig::default();

    if !candidates.is_empty() {
        let selections = prompt_multiselect(
            "Найдены npm-хуки в package.json. Какие выполнять при kley publish?",
            &candidates, // [(name, command), ...], дефолт — ничего не выбрано
        )?; // Vec<bool>

        for ((name, cmd), selected) in candidates.iter().zip(selections) {
            if selected {
                config.hooks.insert(name.to_string(), HookEntry {
                    command: cmd.to_string(),
                });
            }
        }
    }

    // файл создаётся в любом случае — даже пустой `{}`,
    // это и есть маркер "визард уже показывался"
    config.save(hooks_path)?;
    Ok(config)
}
```

Экран визарда:

```
Kley found the following npm lifecycle scripts in package.json.
Select which ones kley should run automatically during `kley publish`:

  PRE-publish (run before copying files):
  [ ] prepare          → npm run build
  [x] prepack          → tsc -p tsconfig.json
  [ ] prepublishOnly   → npm test

  POST-publish (run after copying files):
  [ ] postpublish      → echo "done"

  (space to toggle, enter to confirm)
```

Если `candidates` пуст (в `package.json` нет ни одного из `KNOWN_HOOKS`) — визард не показывает диалог, сразу молча создаёт пустой `.kley/hooks.json` и продолжает публикацию.

Повторный запуск визарда по новым хукам, появившимся в `package.json` после первой публикации — вне рамок v1. Пересмотр решений — через `kley hooks edit` (см. п.4).

---

## 4. CLI-флаги и команды

```bash
kley publish                    # обычный flow, как описано выше
kley publish --non-interactive  # см. п.2
kley publish --no-hooks         # игнорировать .kley/hooks.json на этот раз, чистое копирование
kley hooks list                 # показать текущий .kley/hooks.json в читаемом виде
kley hooks edit                 # пересобрать .kley/hooks.json заново — повторный запуск визарда
                                 #   по всем KNOWN_HOOKS, найденным в package.json
                                 #   (перезаписывает существующий файл целиком)
```

---

## 5. Что осознанно вынесено за рамки v1

- **Guard-rail на опасные команды** (например, `npm publish` внутри хука) — не реализуется в первой версии. Ответственность за содержимое хуков полностью на пользователе, явно подтвердившем каждый через визард. Может быть пересмотрено в будущих версиях (визуальные пометки, более умные проверки, возможно — привлечение агентов для анализа хуков), но не блокирует v1.
- **Git-трекинг `.kley/hooks.json`** (коммитить или добавлять в `.gitignore`) — не регламентируется спецификацией. Kley — локальный инструмент; решение о том, распространять ли конфиг хуков через git, полностью на усмотрение пользователя.
- **Отслеживание изменений текста команды** (hash-проверка при расхождении с `package.json`) — не реализуется. `.kley/hooks.json` — единственный источник истины, `package.json.scripts` используется только как источник кандидатов при первом запуске визарда.

---

## 6. Обработка ошибок

Если любой pre- или post-хук завершается с ненулевым exit code — `kley publish` немедленно прерывается с ошибкой, копирование файлов (или последующие post-хуки) не выполняется.

```
✗ Hook "prepack" failed with exit code 1: tsc -p tsconfig.json
  Publish aborted.
```

Разделение по фазам: если падает **pre**-хук — `copy_package_files` не вызывается вообще. Если падает **post**-хук — файлы уже скопированы (публикация в локальный store формально состоялась), но команда всё равно завершается с ошибкой и ненулевым кодом возврата, чтобы CI/скрипты корректно увидели сбой.

---

## Открытые вопросы для реализации (не блокируют старт)

1. Формат вывода `prompt_multiselect` — использовать существующую CLI-библиотеку (`dialoguer`?) или писать свой рендер.
2. `kley hooks edit` — полностью перезаписывает файл или мержит с существующими выборами как дефолтные значения чекбоксов (чтобы не терять предыдущий выбор при простом просмотре)?
