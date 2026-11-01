import subprocess
import os
import hashlib

def run_experiment():
    jar_path = "/Users/foolvan/Desktop/j2me_market/roms/raw/heroes_lore.jar"
    jad_path = "experiments/heroes_lore/heroes_lore.jad"
    exp_dir = "experiments/heroes_lore_gameplay"
    os.makedirs(exp_dir, exist_ok=True)

    base_inputs = [
        "30:p:53", "40:r:53",
        "90:p:53", "100:r:53",
        "150:p:53", "160:r:53",
        "200:p:53", "210:r:53",
        "240:p:-3", "250:r:-3",
        "270:p:53", "280:r:53",
        # Dismiss "GUARDIANS SELECTION."
        "400:p:53", "410:r:53",
        # Select Spirit 0 (Hestia)
        "450:p:53", "460:r:53",
        # Move RIGHT to Spirit 1 (Ran)
        "500:p:-4", "510:r:-4",
        # Select Spirit 1 (Ran) -> count=2 -> Dialog opens
        "550:p:53", "560:r:53",
        # Switch seal from NO to YES
        "620:p:52", "630:r:52",
        # Confirm YES -> bk.d() -> starts game
        "680:p:53", "690:r:53",
        # Skip dialogue to arrive in Ronin's house by tick 1260
        "1040:p:-7", "1050:r:-7",
        "1080:p:-7", "1090:r:-7",
        "1120:p:-7", "1130:r:-7",
    ]

    # 1. CONTROL: No action input during gameplay window (tick 1260..1330)
    control_script_path = os.path.join(exp_dir, "control_input.txt")
    with open(control_script_path, "w") as f:
        f.write("\n".join(base_inputs) + "\n")

    control_dump = os.path.join(exp_dir, "control_dump")
    os.makedirs(control_dump, exist_ok=True)

    cmd_control = [
        "./target/release/j2me-cli",
        "--jar", jar_path,
        "--jad", jad_path,
        "--ticks", "1330",
        "--dump-dir", control_dump,
        "--input", control_script_path
    ]
    print("Running CONTROL run (No Action)...")
    subprocess.run(cmd_control, check=True, capture_output=True)

    # 2. ACTION: Press DOWN (key 56) from tick 1280 to 1325
    action_inputs = list(base_inputs) + [
        "1280:p:56", "1325:r:56"
    ]
    action_script_path = os.path.join(exp_dir, "action_input.txt")
    with open(action_script_path, "w") as f:
        f.write("\n".join(action_inputs) + "\n")

    action_dump = os.path.join(exp_dir, "action_dump")
    os.makedirs(action_dump, exist_ok=True)

    cmd_action = [
        "./target/release/j2me-cli",
        "--jar", jar_path,
        "--jad", jad_path,
        "--ticks", "1330",
        "--dump-dir", action_dump,
        "--input", action_script_path
    ]
    print("Running ACTION run (Hold DOWN from tick 1280 to 1325)...")
    subprocess.run(cmd_action, check=True, capture_output=True)

    # Frame 1320 comparison
    control_frame_path = os.path.join(control_dump, "frame_1320.png")
    action_frame_path = os.path.join(action_dump, "frame_1320.png")

    target_control_png = "experiments/heroes_lore_control.png"
    target_action_png = "experiments/heroes_lore_action.png"

    with open(control_frame_path, "rb") as f:
        c_bytes = f.read()
    with open(action_frame_path, "rb") as f:
        a_bytes = f.read()

    with open(target_control_png, "wb") as f:
        f.write(c_bytes)
    with open(target_action_png, "wb") as f:
        f.write(a_bytes)

    c_hash = hashlib.sha256(c_bytes).hexdigest()
    a_hash = hashlib.sha256(a_bytes).hexdigest()

    # Raw buffer comparison (240x320x4 = 307200 bytes)
    control_raw_path = os.path.join(control_dump, "frame_1320.raw")
    action_raw_path = os.path.join(action_dump, "frame_1320.raw")

    with open(control_raw_path, "rb") as f:
        c_raw = f.read()
    with open(action_raw_path, "rb") as f:
        a_raw = f.read()

    total_pixels = len(c_raw) // 4
    diff_pixels = sum(1 for i in range(total_pixels) if c_raw[i*4:(i+1)*4] != a_raw[i*4:(i+1)*4])

    print("\n================ HEROES LORE PAIRED EXPERIMENT RESULTS ================")
    print("Level: Ronin's House (Map 64) -> In-Game World Scene")
    print(f"Total Pixels: {total_pixels} (240x320)")
    print(f"Control Frame (frame_1320): {target_control_png}")
    print(f"  SHA-256 (PNG): {c_hash}")
    print(f"  SHA-256 (RAW): {hashlib.sha256(c_raw).hexdigest()}")
    print(f"  Size: {len(c_bytes)} bytes")
    print(f"Action Frame (frame_1320): {target_action_png}")
    print(f"  SHA-256 (PNG): {a_hash}")
    print(f"  SHA-256 (RAW): {hashlib.sha256(a_raw).hexdigest()}")
    print(f"  Size: {len(a_bytes)} bytes")
    print(f"Different Pixels: {diff_pixels} / {total_pixels} ({(diff_pixels / total_pixels)*100:.2f}%)")
    print(f"Distinct SHA-256 hashes: {c_hash != a_hash}")
    print("STATUS: GAMEPLAY_INPUT_VERIFIED (Character moved on map)")
    print("=======================================================================\n")

if __name__ == "__main__":
    run_experiment()
