# Промпт-передача: эмулятор J2ME на Rust — Фаза 3 (ввод, звук, обработка исключений)

Ты продолжаешь работу над эмулятором J2ME на Rust в `/Users/foolvan/Desktop/Rust_j2me_emulator`.
Проект — рабочее пространство Cargo: `j2me-core` (VM, интерпретатор, загрузчик классов, планировщик
зелёных потоков), `j2me-stdlib` (CLDC/MIDP/Nokia/M3D, мост нативных вызовов), `j2me-cli` (headless-прогон
с дампом кадров), `j2me-wasm`.

Тестовые ROM'ы (14 JAR): `/Users/foolvan/Desktop/j2me_market/roms/raw/`
Игра из общего списка 16, которых там НЕТ: `deep_3d`, `stalker_mobile` — не ищи, их нет.

## Правила работы

1. Сборка только через cargo с явным PATH, иначе не находится toolchain:
   `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"`
2. Прогоны игр — только `--release`. Debug-бинарник даёт ~4.5 с/кадр и бесполезен.
3. `cargo fmt --check` в этом воркспейсе шумит диффами по всему дереву — не гоняй его как гейт,
   форматируй только тронутые файлы.
4. Не объявляй фазу закрытой. Критерий — живые потоки, меняющийся кадр, звук и ввод на КАЖДОЙ игре,
   подтверждённые прогоном, а не рассуждением.
5. Числа только из замера. Не оценивай «примерно» — запускай и меряй.
6. Ноутбук тротлит под длительной нагрузкой: не запускай прогоны длиннее ~400 тиков без нужды,
   фоновые прогоны — через `background=true` с `notify=true`.
7. Экономь токены: минимум промежуточных отчётов, сразу к делу.

## Что уже сделано и проверено (НЕ переделывать)

Проверено на этом дереве: `cargo test` → **45 тестов, 0 падений**; `cargo build --release -p j2me-cli` — чисто.

Исправлено и покрыто тестами:

| Файл | Что сделано |
|---|---|
| `crates/j2me-stdlib/src/input.rs` (новый) | Очередь нажатий с авто-повтором, один обработчик за раз |
| `crates/j2me-stdlib/src/emulator.rs` | `key_event` кладёт в очередь; `dispatch_input` в `tick`; сброс очереди при смене displayable |
| `crates/j2me-stdlib/src/lib.rs` | `pub mod input` |
| `crates/j2me-core/src/vm/interpreter.rs` | `handle_athrow` теперь матчит `catch_type` по иерархии классов, а не ловит первым попавшимся |
| `crates/j2me-core/src/classfile/attributes.rs` | `ExceptionTableEntry.catch_class` — имя класса резолвится при парсинге |
| `crates/j2me-core/src/vm/class_loader.rs` | `BUILTIN_SUPERS` — иерархия `java/lang/*Exception`, `java/io/*`, `javax/microedition/rms/*` для синтезированных классов |
| `crates/j2me-stdlib/src/bridge/native_bridge.rs` | Реализованы `RecordStore.enumerateRecords` + `RecordEnumeration` (были ОТСУТСТВОВАЛИ → null → NPE); исправлен `String.indexOf(String)` (байтовый срез внутри UTF-8 символа + неверный возврат индекса) |
| `crates/j2me-stdlib/src/midp/rms/record_store.rs` | `ids()` — перечисление в возрастающем порядке + тесты |
| `crates/j2me-stdlib/src/media/wav.rs` | Валидация формата (PCM/8/16 бит), честная ошибка на остальное + тесты |
| `crates/j2me-stdlib/src/media/midi.rs` | `media_time_ms()` / `seek_ms()` + тесты |
| `crates/j2me-stdlib/src/media/manager.rs` | `wav_play_head` сбрасывается в `start()` (звук дублировался/продолжался с середины) + тесты |
| `crates/j2me-stdlib/src/bridge/context.rs` | Тесты на залипание `getKeyStates` |
| `crates/j2me-cli/src/main.rs` | Флаг `--audio-dump <path.wav>` — выгрузка реального аудио |

Звук подтверждён на живом прогоне (`bounce_tales`): 1.26 с, 96% слышимых сэмплов, пик 0.772, без клиппинга.
Ввод подтверждён на `gravity_defied`: кадры на 700-м тике различаются при нажатии FIRE и без него.

## Текущее состояние матрицы (14 игр, 120 тиков, release)

Живой поток (`Threads: 1`) — 9 игр: `bounce_tales`, `castlevania_aria`, `cyberpunk`, `doom_rpg`,
`god_of_war`, `gravity_defied`, `prince_sands`, `sc_chaos_theory`, `wolfenstein_rpg`.
Богатый кадр (101 цвет): `gravity_defied`, `heroes_lore`, `prince_sands`.

Поток мёртв (`Threads: 0`) — 5 игр: `assassins_creed`, `heroes_lore`, `nfs_most_wanted`, `re_deg`, `sh_orphan`.

## Блокеры по приоритету — это и есть твоя работа

### Блокер 1. `bounce_tales` — 312 мс/кадр (неиграбельно)

Замер: 60 тиков за **18.73 с**. Для сравнения на том же бинарнике: `prince_sands` 60 тиков за 0.06 с,
`gravity_defied` 0.03 с, `wolfenstein_rpg` 0.05 с. То есть проблема локальная в этой игре, а не общая
скорость VM — ищи горячий путь, вероятнее всего синтез MIDI/рендер звука за тик (игра грузит `ax.mid`…`bd.mid`).

Как мерить:
```
/usr/bin/time -p ./target/release/j2me-cli "/Users/foolvan/Desktop/j2me_market/roms/raw/bounce_tales.jar" --ticks 60 --dump-dir target/audit/perf
```
Цель: ≤ 16 мс/кадр (60 FPS), минимум — ≤ 50 мс/кадр.

### Блокер 2. Два непойманных NPE

- `nfs_most_wanted`: `Thread 'main' unhandled exception: java/lang/NullPointerException (ObjectId(147))`
- `sh_orphan`: `Thread 'GameMidlet-run' unhandled exception: java/lang/NullPointerException (ObjectId(2435))`

NPE теперь корректно матчится по иерархии, значит эти игры падают там, где обработчика нет вовсе —
то есть на реально отсутствующем API или на возвращённом null из нативного метода.
Найди, какой именно вызов возвращает null. Есть готовые отладочные флаги (включаются переменной окружения):
`J2ME_DEBUG_RMS`, `J2ME_DEBUG_PROPS`, `J2ME_DEBUG_INPUT`, `J2ME_DEBUG_STATE`, `J2ME_DEBUG_THREADS`.
Добавь по аналогии точечный лог на подозреваемый нативный метод и сними трассу — не гадай.

### Блокер 3. Пять игр с мёртвым потоком

`assassins_creed`, `heroes_lore`, `nfs_most_wanted`, `re_deg`, `sh_orphan`.
Важно: у `heroes_lore` и `assassins_creed` при этом рисуется богатый кадр (101 цвет) — значит сплэш
отработал, а игровой цикл умер позже. Отличие от «пустого экрана» принципиально, не путай эти случаи.
Определи для каждой игры точку смерти потока и общий корень, если он один.

### Блокер 4. JAD не поддерживается вообще

В коде нет ни одного упоминания JAD (`grep -rli jad crates/*/src` — пусто), файлов `.jad` в каталоге ROM нет.
Игры, читающие `getAppProperty` из JAD, получают null. Нужно: парсер JAD + слияние свойств
JAD/манифест с приоритетом JAD, как в спецификации MIDP.

### Блокер 5. Остатки отладки

В коде остались env-гейтованные отладочные принты: `J2ME_DEBUG_PROPS`, `J2ME_DEBUG_RMS` (native_bridge.rs),
`J2ME_DEBUG_STATE`, `J2ME_DEBUG_INPUT` (emulator.rs), `J2ME_DEBUG_THREADS` (scheduler.rs).
Либо приведи к единому виду и задокументируй, либо убери.

## Как проверять результат

Матрица по всем играм (создай скрипт сам, образец логики):
```bash
for g in bounce_tales cyberpunk god_of_war doom_rpg gravity_defied wolfenstein_rpg \
         sc_chaos_theory prince_sands assassins_creed castlevania_aria sh_orphan \
         nfs_most_wanted heroes_lore re_deg; do
  ./target/release/j2me-cli "/Users/foolvan/Desktop/j2me_market/roms/raw/$g.jar" \
    --ticks 120 --dump-dir target/audit/run/$g > target/audit/run/$g.log 2>&1
  echo "$g $(grep -o 'Threads: [0-9]*' target/audit/run/$g.log | tail -1) \
$(grep -o 'colors: [0-9]*' target/audit/run/$g.log | tail -1) \
err=$(grep -c 'VM ERROR' target/audit/run/$g.log)"
done
```

Признаки успеха для каждой игры: `Threads: 1`, `colors` растёт от кадра к кадру, `err=0`,
и кадр МЕНЯЕТСЯ при подаче ввода:
```bash
--input "360:down:FIRE,395:up:FIRE"   # формат tick:action:key, НЕ action:tick
```
Сверяй кадры хешем: `md5 -q target/audit/run/<game>/frame_390.png target/audit/run/<game>/frame_400.png`.

Ввод, звук и картинку проверяй на каждой игре отдельно — общий «зелёный» тест ничего не доказывает.
Отчёт: что реально замерено, что осталось, какой блокер следующий. Фазу 3 не закрывай.
