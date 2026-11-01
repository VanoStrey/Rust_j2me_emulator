import os
import subprocess

def main():
    jar_path = "/Users/foolvan/Desktop/j2me_market/roms/raw/heroes_lore.jar"
    jad_path = "/Users/foolvan/Desktop/Rust_j2me_emulator/experiments/heroes_lore/heroes_lore.jad"
    dump_dir = "/Users/foolvan/Desktop/Rust_j2me_emulator/experiments/heroes_lore/level_dump"
    os.makedirs(dump_dir, exist_ok=True)

    # 1. Splash at tick 200 -> press 53 (FIRE)
    # 2. Character creation at tick 340 -> press 53 (FIRE) to select Ronin
    # 3. Popup appears ("YES NO", cursor on NO).
    #    Tick 440: press 52 (LEFT) -> cursor moves to YES!
    #    Tick 480: press 53 (FIRE) -> confirms YES!
    # 4. Now Guardian Spirit selection screen appears!
    #    Tick 580: press 53 (FIRE) -> selects Spirit 0 (count=1)
    #    Tick 620: press 54 (RIGHT) -> moves cursor to Spirit 1
    #    Tick 660: press 53 (FIRE) -> selects Spirit 1 (count=2 -> Dialog opens!)
    #    Tick 720: press 52 (LEFT) -> flips confirmation dialog to YES!
    #    Tick 760: press 53 (FIRE) -> confirms dialog -> d() called -> n.a() called!
    # 5. Tick 850..1300: game loads map 0, starts dialogue/gameplay!
    #    We press 53 periodically to advance dialogue to live map.
    #    Tick 860, 930, 1000, 1070, 1140, 1210: press 53
    # 6. At tick 1300: Control frame!
    # 7. Action: press and hold DOWN (key 56) from tick 1310 to 1420.
    # 8. At tick 1450: Action frame!

    script_path = "/Users/foolvan/Desktop/Rust_j2me_emulator/experiments/heroes_lore/input_level.txt"
    lines = [
        # Title -> Char creation
        "200:KEY_PRESS:53",
        "220:KEY_RELEASE:53",
        # Char creation -> Ronin
        "340:KEY_PRESS:53",
        "360:KEY_RELEASE:53",
        # Ronin dialog -> YES
        "440:KEY_PRESS:52",
        "460:KEY_RELEASE:52",
        "490:KEY_PRESS:53",
        "510:KEY_RELEASE:53",
        # Spirit 0 select
        "590:KEY_PRESS:53",
        "610:KEY_RELEASE:53",
        # Move right to Spirit 1
        "650:KEY_PRESS:54",
        "670:KEY_RELEASE:54",
        # Spirit 1 select -> dialog opens!
        "710:KEY_PRESS:53",
        "730:KEY_RELEASE:53",
        # Flip spirit dialog to YES
        "780:KEY_PRESS:52",
        "800:KEY_RELEASE:52",
        # Confirm spirit dialog -> start game!
        "840:KEY_PRESS:53",
        "860:KEY_RELEASE:53",
        # Advance dialogue lines
        "940:KEY_PRESS:53",
        "960:KEY_RELEASE:53",
        "1040:KEY_PRESS:53",
        "1060:KEY_RELEASE:53",
        "1140:KEY_PRESS:53",
        "1160:KEY_RELEASE:53",
        "1240:KEY_PRESS:53",
        "1260:KEY_RELEASE:53",
        "1340:KEY_PRESS:53",
        "1360:KEY_RELEASE:53",
        # Action movement: DOWN (key 56)
        "1420:KEY_PRESS:56",
        "1500:KEY_RELEASE:56"
    ]
    with open(script_path, "w") as f:
        f.write("\n".join(lines) + "\n")

    cmd = [
        "./target/release/j2me-cli",
        "--jar", jar_path,
        "--jad", jad_path,
        "--ticks", "1550",
        "--dump-dir", dump_dir,
        "--input-script", script_path
    ]
    print("Running Heroes Lore Level driving test...")
    proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    print(f"Exit code: {proc.returncode}")
    frames = sorted([f for f in os.listdir(dump_dir) if f.startswith("frame_") and f.endswith(".png")])
    print(f"Total dumped frames: {len(frames)}")
    for f in frames[-15:]:
        p = os.path.join(dump_dir, f)
        print(f"  {f}: {os.path.getsize(p)} bytes")

if __name__ == "__main__":
    main()
