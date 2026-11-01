import os
import sys
import subprocess
import json
import hashlib

def get_sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()

def main():
    jar_path = "/Users/foolvan/Desktop/j2me_market/roms/raw/heroes_lore.jar"
    jad_path = "/Users/foolvan/Desktop/Rust_j2me_emulator/experiments/heroes_lore/heroes_lore.jad"
    dump_dir = "/Users/foolvan/Desktop/Rust_j2me_emulator/experiments/heroes_lore/gameplay_dump"
    os.makedirs(dump_dir, exist_ok=True)

    # Let's write an input script that:
    # 1. Waits past splash (tick 200), presses 53 (FIRE) to enter character creation
    # 2. At tick 320, presses 53 (FIRE) on character creation (Ronin selected)
    # 3. At tick 400, presses 53 (FIRE) on by.class confirmation
    # 4. At tick 500, presses 53 (FIRE) on bk.class (Guardian Spirit selected -> opens dialog)
    # 5. At tick 580, presses 52 (LEFT) to toggle dialog cursor to Yes (d = true)
    # 6. At tick 650, presses 53 (FIRE) to confirm and enter game map!
    # 7. Then at tick 750+, game enters map/intro dialogue.
    # 8. Press 53 multiple times (tick 750, 820, 890, 960) to advance any initial story dialogue to world map!
    # 9. Control frame at tick 1100.
    # 10. Action: hold DOWN (key 56 or key -2) from tick 1110 to 1180.
    # 11. Action frame at tick 1200.

    script_path = "/Users/foolvan/Desktop/Rust_j2me_emulator/experiments/heroes_lore/gameplay_input.txt"
    lines = [
        # Tick 200: Fire on Title
        "200:KEY_PRESS:53",
        "220:KEY_RELEASE:53",
        # Tick 340: Fire on Character Selection
        "340:KEY_PRESS:53",
        "360:KEY_RELEASE:53",
        # Tick 440: Fire on by.class
        "440:KEY_PRESS:53",
        "460:KEY_RELEASE:53",
        # Tick 540: Fire on bk.class (Spirit)
        "540:KEY_PRESS:53",
        "560:KEY_RELEASE:53",
        # Tick 620: LEFT on dialog (toggle to Yes)
        "620:KEY_PRESS:52",
        "640:KEY_RELEASE:52",
        # Tick 700: Fire on dialog (confirm start game)
        "700:KEY_PRESS:53",
        "720:KEY_RELEASE:53",
        # Advance dialogue lines
        "820:KEY_PRESS:53",
        "840:KEY_RELEASE:53",
        "920:KEY_PRESS:53",
        "940:KEY_RELEASE:53",
        "1020:KEY_PRESS:53",
        "1040:KEY_RELEASE:53",
        "1120:KEY_PRESS:53",
        "1140:KEY_RELEASE:53",
        # Action movement: hold DOWN (56 / 8 / -2)
        "1220:KEY_PRESS:56",
        "1280:KEY_RELEASE:56",
        "1320:KEY_PRESS:56",
        "1380:KEY_RELEASE:56"
    ]
    with open(script_path, "w") as f:
        f.write("\n".join(lines) + "\n")

    cmd = [
        "./target/release/j2me-cli",
        "--jar", jar_path,
        "--jad", jad_path,
        "--ticks", "1450",
        "--dump-dir", dump_dir,
        "--input-script", script_path
    ]
    print("Running Heroes Lore driving test...")
    proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    print(f"Exit code: {proc.returncode}")
    
    # List dumped frames
    frames = sorted([f for f in os.listdir(dump_dir) if f.startswith("frame_") and f.endswith(".png")])
    print(f"Total dumped frames: {len(frames)}")
    for f in frames[-15:]:
        p = os.path.join(dump_dir, f)
        print(f"  {f}: {os.path.getsize(p)} bytes")

if __name__ == "__main__":
    main()
