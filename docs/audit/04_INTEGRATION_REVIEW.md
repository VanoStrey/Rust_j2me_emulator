# 04. Интеграционный обзор (Integration Review)
## Взаимодействие J2ME Market и Rust J2ME Emulator: Контракт, Память и WebAssembly

---

## 1. Спецификация WebAssembly C-ABI контракта

Интерфейс между браузерным JavaScript и Rust-ядром реализован через C-совместимые экспортируемые функции (`extern "C"`) в крейте `crates/j2me-wasm/src/lib.rs`.

### Экспортируемые функции WASM API:

| Функция | Сигнатура | Назначение |
|---|---|---|
| `emu_init` | `(width: u32, height: u32, sample_rate: u32)` | Инициализация нового экземпляра эмулятора |
| `emu_alloc_buffer` | `(size: usize) -> *mut u8` | Выделение буфера в памяти WASM (до 64 МБ) |
| `emu_free_buffer` | `(ptr: *mut u8, size: usize)` | Безопасное освобождение буфера с проверкой таблицы аллокаций |
| `emu_load_jar` | `(ptr: *const u8, len: usize) -> i32` | Загрузка JAR-файла из памяти |
| `emu_load_jar_with_filename` | `(jar_ptr, jar_len, name_ptr, name_len) -> i32` | Загрузка JAR с передачей оригинального имени для детектора разрешения |
| `emu_load_jad` | `(ptr: *const u8, len: usize) -> i32` | Загрузка дескриптора JAD |
| `emu_tick` | `(delta_seconds: f64) -> i32` | Исполнение одного кадра виртуальной машины |
| `emu_get_framebuffer_ptr` | `() -> *const u8` | Указатель на плоский массив RGBA8888 кадрового буфера |
| `emu_get_framebuffer_len` | `() -> usize` | Длина кадрового буфера в байтах |
| `emu_get_screen_width` | `() -> u32` | Текущая ширина экрана игры |
| `emu_get_screen_height` | `() -> u32` | Текущая высота экрана игры |
| `emu_key_event` | `(keycode: i32, pressed: i32)` | Передача событий клавиатуры (нажатие / отпускание) |
| `emu_read_audio_samples` | `(dest_ptr: *mut f32, max_samples: usize) -> usize` | Чтение синтезированных аудиосемплов Float32 |
| `emu_get_audio_queue_len` | `() -> usize` | Длина очереди ожидающих аудиосемплов |
| `emu_export_rms` | `() -> *const u8` | Сериализация баз сохранений RMS в бинарный буфер |
| `emu_import_rms` | `(ptr: *const u8, len: usize) -> i32` | Десериализация и восстановление сохранений RMS |
| `emu_set_strict_mode` | `(strict: i32)` | Включение/отключение строгого режима исполнения (без Auto-Healing) |
| `emu_is_strict_mode` | `() -> i32` | Проверка статуса строгого режима |

---

## 2. Управление памятью и Zero-Copy Framebuffer

### 2.1. Концепция Zero-Copy
В классических эмуляторах кадровый буфер часто передается в JavaScript через копирование байтов (`postMessage` или вызовы с созданием промежуточных массивов).  
В данной системе реализован **истинный Zero-Copy пайплайн**:
1. Rust-ядро аллоцирует непрерывный вектор `Vec<u32>` кадрового буфера в своей линейной памяти WebAssembly.
2. Функция `emu_get_framebuffer_ptr()` возвращает линейный адрес указателя на первый байт буфера.
3. JavaScript-код в `WasmBridge.js` создает прямой типизированный срез поверх разделяемой памяти WebAssembly:
   ```javascript
   const ptr = window.J2MEWasm.emu_get_framebuffer_ptr();
   const rawBuffer = new Uint8ClampedArray(window.J2MEWasm.memory.buffer, ptr, fbSize);
   const imgData = new ImageData(rawBuffer, curW, curH);
   offscreenCtx.putImageData(imgData, 0, 0);
   ```
4. Блиттинг на HTML5 Canvas выполняется напрямую без единого промежуточного копирования на уровне движка JavaScript V8/JSC!

### 2.2. Защита от инвалидации буфера при росте памяти (Memory Growth)
Если в процессе работы эмулятора WebAssembly увеличивает память (`memory.grow()`), существующий `ArrayBuffer` в JavaScript автоматически переходит в состояние `detached` (отсоединен).  
**Проверка кода:** `WasmBridge.js` в каждом кадре заново обращается к геттеру `window.J2MEWasm.memory.buffer`. Это гарантирует, что при любом расширении кучи создается актуальный `Uint8ClampedArray` и обращение к памяти никогда не выбрасывает исключение `TypeError: Cannot perform %TypedArray%.prototype.set on a detached ArrayBuffer`.

### 2.3. Устранение утечки аудио-буфера (Дефект P1 — УСТРАНЕН)
В ходе аудита было установлено, что метод `J2MEWasm.emu_load_jar` в файле `market_ui/wasm/j2me_wasm.js` при каждом запуске новой игры заново аллоцировал 16 КБ буфер аудиосемплов (`audioBufferPtr = wasmExports.emu_alloc_buffer(4096 * 4)`), не освобождая ранее выделенный указатель.  
**Исправление:** Добавлена явная проверка `if (audioBufferPtr) wasmExports.emu_free_buffer(audioBufferPtr, 4096 * 4)`. Теперь утечка линейной памяти при многократной смене игр исключена.

---

## 3. Звуковой пайплайн (WebAudio Mastering Chain)

Звуковой тракт (`market_ui/wasm/j2me_wasm.js`) спроектирован по студийным стандартам цифровой обработки сигналов:

1. **Hardware-Pull Audio Processing:**
   - Основной режим использует `AudioWorkletNode` (`j2me_audio_worklet.js`), работающий в выделенном низколатентном звуковом потоке операционной системы.
   - Для браузеров без поддержки модулей ворклетов сохранен автоматический фолбэк на `ScriptProcessorNode` (размер буфера 2048 семплов = 46.4 мс при 44.1 кГц).
2. **Студийная цепочка мастеринга:**
   - **High-Pass DC Blocking Filter:** Фильтр Баттерворта 2-го порядка с частотой среза 35 Гц и добротностью Q=0.7071 для полного устранения постоянного смещения тока (DC offset) и предотвращения гула в динамиках.
   - **Soft-Limiter & Dynamics Compressor:** Прозрачный лимитер с порогом -6 dB, крутизной 14:1 и временем атаки 2 мс для подавления перегрузок при одновременном звучании полифонии.
   - **Zero-Click Smooth Fading:** При нехватке семплов (buffer underrun) применяется сглаживающее угасание амплитуды за 16 семплов, полностью исключающее характерные цифровые щелчки при паузе или остановке звука.

---

## 4. Синхронизация состояний и версионирование бинарников

В ходе аудита было обнаружено расхождение между бинарными файлами в репозитории эмулятора и маркета.  
После пересборки актуального состояния ядра с исправлением строгого режима и защитой от утечек памяти хеши были полностью синхронизированы:

```text
Target WASM Release:   52433985dba3df6cb73a2b56ffcba64965280c1d63f20766229bf27f24e8cc84
Emulator web/:         52433985dba3df6cb73a2b56ffcba64965280c1d63f20766229bf27f24e8cc84
Market UI wasm/:       52433985dba3df6cb73a2b56ffcba64965280c1d63f20766229bf27f24e8cc84
Статус: СИНХРОНИЗИРОВАНО (100% PARITY)
```
