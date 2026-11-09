# 13. Журнал верификационных испытаний (Verification Log)
## Протокол выполненных команд, замеров, хешей и тестовых запусков

---

## 1. Исходные состояния репозиториев (9 ноября 2026 г., 03:03 MSK)

### Репозиторий J2ME Market:
- **HEAD Commit:** `2dc8b5642a9188d21800521cde156e869e3bff72`
- **Сообщение:** `feat(audio, compatibility, pwa): add retro MIDI jukebox, Web Gamepad API, offline PWA and compatibility matrix`
- **Ветка:** `main`, clean working tree (submodule tools/freej2me).

### Репозиторий Rust J2ME Emulator:
- **HEAD Commit:** `0c1de93bca7a54862acae82ba59aec5376fca280`
- **Состояние дерева:** 79 измененных файлов (рабочее состояние с оптимизациями рендеринга и аудио).

---

## 2. Протокол проверки и синхронизации бинарных файлов WebAssembly

### Исходные хеши (до синхронизации):
- `market_ui/wasm/j2me_wasm.wasm`: `b8d0e34522c2c6acf2ea24e55318917a23e4fb593d3cf3b5b5bb3aa37f90751b` (2,825,047 байт)
- `Rust_j2me_emulator/web/j2me_wasm.wasm`: `1f5f42a2f4b2961b6dfe83efce221353c4aca8f66d5bd727c4f1d8de11438066` (2,824,946 байт)
- **Вывод:** Обнаружено расхождение версий.

### Компиляция и финальная синхронизация:
- **Команда сборки:**
  ```bash
  PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" \
  cargo build --manifest-path /Users/foolvan/Desktop/Rust_j2me_emulator/Cargo.toml \
  -p j2me-wasm --target wasm32-unknown-unknown --release
  ```
- **Финальный согласованный SHA-256 хеш:**
  ```text
  52433985dba3df6cb73a2b56ffcba64965280c1d63f20766229bf27f24e8cc84  Rust_j2me_emulator/web/j2me_wasm.wasm
  52433985dba3df6cb73a2b56ffcba64965280c1d63f20766229bf27f24e8cc84  j2me_market/market_ui/wasm/j2me_wasm.wasm
  ```

---

## 3. Протокол тестирования виртуальной машины на играх (Release Runner)

Команда: `target/release/j2me-cli <path> --ticks 180 --no-input`

1. **Bounce Tales (`bounce_tales.jar`):**
   - Разрешение: 240x320 (FALLBACK_DEFAULT).
   - Результат: 180 тиков за 96.4 мс = **1,867.48 ticks/s**.
   - Framebuffer Publishes: 358.
2. **God of War: Betrayal (`god_of_war.jar`):**
   - Разрешение: 240x320 (MANIFEST).
   - Результат: 180 тиков за 2,801.3 мс = **64.26 ticks/s**.
   - Framebuffer Publishes: 13.
3. **Cyberpunk: The Arasaka's Plot (`cyberpunk.jar`):**
   - Разрешение: 240x320 (MANIFEST).
   - Результат: 180 тиков за 76.8 мс = **2,343.22 ticks/s**.
   - Paint Calls: 35. Framebuffer Publishes: 35.
4. **Doom RPG (`doom_rpg.jar`):**
   - Разрешение: 128x128 (ASSET_INSPECTION).
   - Результат: 180 тиков за 14.9 мс = **12,089.12 ticks/s**.
5. **Castlevania: Order of Shadows (`castlevania_oos.jar`):**
   - Разрешение: 176x103 (ASSET_INSPECTION).
   - Результат: 180 тиков за 30.3 мс = **5,950.47 ticks/s**.
   - Paint Calls: 40. Активных потоков: 2.
6. **Allods (`allods.jar`):**
   - Разрешение: 240x320 (FALLBACK_DEFAULT).
   - Результат: 180 тиков за 233.0 мс = **772.53 ticks/s**.
   - Paint Calls: 45.
7. **Gravity Defied (`gravity_defied.jar`):**
   - Разрешение: 240x320 (FALLBACK_DEFAULT).
   - Результат: 180 тиков за 46.2 мс = **3,891.94 ticks/s**.
   - Paint Calls: 31.

---

## 4. Протокол автоматических регрессионных тестов

1. **`cargo test --manifest-path Rust_j2me_emulator/Cargo.toml`:**
   - 202 теста пройдено, 0 провалено. Время выполнения: 3.48 с.
2. **`cargo test --manifest-path j2me_market/server_rust/Cargo.toml`:**
   - 6 тестов пройдено, 0 провалено (включая `test_ssr_xss_prevention_in_title_and_meta`).
3. **`python3 scripts/test_unit.py` (J2ME Market):**
   - 11 тестов пройдено, 0 провалено (включая проверку аннотаций типов).
