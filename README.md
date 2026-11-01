# Rust J2ME Emulator (KVM & MIDP 2.0 / Nokia UI / MMAPI)

Высокопроизводительный, модульный, автономный эмулятор платформы **Java 2 Micro Edition (CLDC 1.1 / MIDP 2.0)**, полностью спроектированный и реализованный с нуля на языке **Rust**.

Компилируется как в нативный бинарник (CLI-раннер, headless-тесты, бенчмаркинг), так и в **WebAssembly (`wasm32-unknown-unknown`)** с прямым доступом к общей памяти (Zero-Copy Shared Framebuffer) для работы в браузере со стабильными 60 FPS.

---

## Архитектура Workspace (Cargo Crates)

```
Rust_j2me_emulator/
├── Cargo.toml                         # Корневой workspace
├── crates/
│   ├── j2me-core/                     # no_std / чистый Rust: ядро VM и парсеры
│   │   ├── src/classfile/             # JVM ClassFile, ConstantPool, Attributes, Preverification
│   │   ├── src/jar/                   # In-Memory ZIP Reader & MANIFEST.MF parser
│   │   ├── src/heap/                  # SlotMap / Arena Heap, ObjectId, Mark-and-Sweep GC
│   │   ├── src/threads/               # Детерминированный Green Threads Scheduler
│   │   └── src/vm/                    # Pre-decoded Instruction Stream & 200+ JVM Opcode Interpreter
│   ├── j2me-stdlib/                   # Стандартные библиотеки и нативный мост (CLDC 1.1 + MIDP 2.0)
│   │   ├── src/bridge/                # NativeBridge JNI Interceptor & VmContext
│   │   ├── src/cldc/                  # java.lang, java.io, Windows-1251 / UTF-8 / ISO encodings
│   │   ├── src/midp/                  # lcdui (Graphics, Image, Font), Game API (Sprite), RMS
│   │   ├── src/nokia/                 # com.nokia.mid.ui (FullCanvas, DirectGraphics, DeviceControl)
│   │   ├── src/media/                 # MMAPI (JSR-135): Manager, Player, PCM Tone Synthesizer
│   │   ├── src/m3d/                   # 3D Stubs: Mascot Capsule v3 & JSR-184 M3G
│   │   └── src/emulator.rs            # Главный координатор виртуальной машины
│   ├── j2me-wasm/                     # Zero-Copy C-ABI / WASM экспорты для браузера
│   └── j2me-cli/                      # Headless CLI раннер и верификационный тест-сьют
└── web/
    ├── index.html                     # Ретро-интерфейс мобильного телефона с 60 FPS канвасом
    └── j2me_wasm.wasm                 # Скомпилированный WASM бинарник (404 КБ)
```

---

## Ключевые инженерные решения

1. **Pre-decoded Threaded Instruction Stream**:
   Байткод каждого метода парсится и предвычисляется один раз при загрузке. Все смещения `goto`/переходов преобразуются в прямые индексы инструкций, а индексы `ConstantPool` — в разрешенные строковые/числовые значения. Это устраняет строковые поиски в горячем цикле интерпретатора!
2. **Кооперативный планировщик зеленых потоков (Green Threads Scheduler)**:
   Поскольку стандартный браузерный WASM выполняется в одном потоке, эмулятор реализует детерминированный планировщик потоков внутри самой VM. Потоки квантуются по 5 000 инструкций и моментально отдают управление при `Thread.sleep()`, `yield()`, `wait()`.
3. **Non-Strict PNG CRC Decoder (Palette Swapping Hack)**:
   Многие игры 2004–2008 годов (Gameloft, EA Mobile, Glu) динамически переписывали байты `PLTE` и `tRNS` чанков в памяти без пересчета контрольной суммы CRC32/Adler32. Наш PNG-декодер игнорирует CRC32 чанков и восстанавливает палитру без падений.
4. **Zero-Copy Shared Framebuffer**:
   Пиксельный буфер `Vec<u32>` в формате `RGBA8888` транслируется в JS без единой аллокации через `new ImageData(new Uint8ClampedArray(wasm.memory.buffer, ptr, len), width, height)`.
5. **Встроенный растровый шрифтовой движок**:
   Глифы 8x8 с полной поддержкой ASCII + кириллицы (Windows-1251, `А-Я`, `а-я`, `Ё`, `ё`), масштабируемые для трех размеров (`SMALL`, `MEDIUM`, `LARGE`) и стилей (`PLAIN`, `BOLD`, `ITALIC`, `UNDERLINED`).
6. **Полноценная реализация Nokia UI DirectGraphics**:
   Растеризация полигонов (`fillPolygon`, `drawPolygon`), прямое манипулирование пикселями (`drawPixels` для `TYPE_INT_8888_ARGB`), побитовые повороты и зеркалирования (`FLIP_HORIZONTAL`, `FLIP_VERTICAL`, `ROTATE_90/180/270`).
7. **MMAPI & RMS**:
   - Синтезатор синусоидальных тонов по формуле $f = 440 \times 2^{(note - 69) / 12}$ и кольцевой PCM буфер для Web Audio API.
   - Двунаправленная бинарная сериализация RecordStore для синхронизации сохранений с `IndexedDB` / `localStorage`.

---

## Сборка и тестирование

### 1. Запуск верификационного тест-сьюта (11 тестов из 11):
```bash
cargo run --release -p j2me-cli -- --test
```

### 2. Запуск headless-раннера для `.jar` с поддержкой JAD, аудио-дампа и скрипта ввода:
```bash
cargo run --release -p j2me-cli -- /path/to/game.jar \
  --ticks 120 \
  --dump-dir target/audit/run \
  --audio-dump target/audit/audio.wav \
  --input "60:down:FIRE,90:up:FIRE"
```
- **JAD Descriptor**: если рядом с `.jar` лежит одноименный `.jad`, он подхватывается автоматически с соблюдением приоритета MIDP (JAD переопределяет MANIFEST.MF). Также можно указать явно: `--jad /path/to/file.jad`.
- **Audio Dump**: сохраняет 16-bit PCM WAV файл и выводит статистику: число сэмплов, частоту, % слышимого сигнала, пиковую амплитуду и клиппинг.
- **Input Script**: формат `tick:action:key`, поддерживаются `FIRE`, `UP`, `DOWN`, `LEFT`, `RIGHT`, `SOFT1`, `SOFT2`, `0`..`9`, `*`, `#`.

### 3. Переменные окружения для точечной отладки (Debug Flags):
- `J2ME_DEBUG_THREADS=1`: трейсинг зеленых потоков (спавн, квантование, `Thread.sleep()`, завершение) в планировщике `Scheduler`.
- `J2ME_DEBUG_STATE=1`: отслеживание жизненного цикла эмулятора и переключения `Displayable`.
- `J2ME_DEBUG_INPUT=1`: подробный лог очереди ввода, физических состояний клавиш и вызовов `keyPressed`/`keyReleased`.
- `J2ME_DEBUG_PROPS=1`: трассировка чтения свойств MIDlet через `getAppProperty` (приоритет JAD > Manifest).
- `J2ME_DEBUG_RMS=1`: детальный аудит операций Record Management System (открытие хранилищ, чтение/запись записей, перечисления).

### 4. Компиляция в WebAssembly:
```bash
cargo build -p j2me-wasm --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/j2me_wasm.wasm web/
```

### 5. Запуск веб-эмулятора в браузере:
Откройте локальный сервер в папке `web/`:
```bash
python3 -m http.server 8080 --directory web
```
И перейдите по адресу: [http://localhost:8080](http://localhost:8080).
Вы можете перетащить любой `.jar` файл в зону дропа для немедленного запуска!
