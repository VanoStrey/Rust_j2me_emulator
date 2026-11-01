import os
import subprocess

def main():
    jar_path = "/Users/foolvan/Desktop/j2me_market/roms/raw/heroes_lore.jar"
    jad_path = "/Users/foolvan/Desktop/Rust_j2me_emulator/experiments/heroes_lore/heroes_lore.jad"
    dump_dir = "/Users/foolvan/Desktop/Rust_j2me_emulator/experiments/heroes_lore/step2_dump"
    os.makedirs(dump_dir, exist_ok=True)

    # 1. Splash at tick 200 -> press 53 (FIRE)
    # 2. Character creation at tick 340 -> press 53 (FIRE) to select Ronin
    # 3. Popup appears ("YES NO", cursor on NO).
    #    Tick 440: press 52 (LEFT) -> cursor moves to YES!
    #    Tick 480: press 53 (FIRE) -> confirms YES!
    # 4. Now Guardian Spirit selection screen appears!
    #    Tick 580: press 53 (FIRE) -> selects spirit
    #    Tick 640: popup appears for spirit ("YES NO").
    #    Tick 680: press 52 (LEFT) -> cursor moves to YES!
    #    Tick 720: press 53 (FIRE) -> confirms spirit!
    # 5. Game begins!

    script_path = "/Users/foolvan/Desktop/Rust_j2me_emulator/experiments/heroes_lore/input_step2.txt"
    lines = [
        "200:KEY_PRESS:53",
        "220:KEY_RELEASE:53",
        "340:KEY_PRESS:53",
        "360:KEY_RELEASE:53",
        "440:KEY_PRESS:52",
        "460:KEY_RELEASE:52",
        "500:KEY_PRESS:53",
        "520:KEY_RELEASE:53",
        "600:KEY_PRESS:53",
        "620:KEY_RELEASE:53",
        "680:KEY_PRESS:52",
        "700:KEY_RELEASE:52",
        "740:KEY_PRESS:53",
        "760:KEY_RELEASE:53",
    ]
    with open(script_path, "w") as f:
        f.write("\n".join(lines) + "\n")

    cmd = [
        "./target/release/j2me-cli",
        "--jar", jar_path,
        "--jad", jad_path,
        "--ticks", "850",
        "--dump-dir", dump_dir,
        "--input-script", script_path
    ]
    subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    frames = sorted([f for f in os.listdir(dump_dir) if f.startswith("frame_") and f.endswith(".png")])
    print(f"Total dumped frames: {len(frames)}")
    for f in frames[-10:]:
        p = os.path.join(dump_dir, f)
        print(f"  {f}: {os.path.getsize(p)} bytes")

if __name__ == "__main__":
    main()
