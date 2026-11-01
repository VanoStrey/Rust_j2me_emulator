import subprocess
import os
import json

def run_scenario(name, input_script, ticks=360):
    script_path = f"/tmp/{name}_input.txt"
    with open(script_path, "w") as f:
        f.write(input_script)
        
    out_dir = f"dumps/{name}_frames"
    os.makedirs(out_dir, exist_ok=True)
    
    cmd = [
        "./target/release/j2me-cli",
        "--jar", "/Users/foolvan/Desktop/j2me_market/roms/Games/CastlevaniaAoS.jar",
        "--ticks", str(ticks),
        "--dump-dir", out_dir,
        "--input", script_path
    ]
    res = subprocess.run(cmd, capture_output=True, text=True)
    print(f"[{name}] return code:", res.returncode)
    for line in res.stdout.splitlines():
        if "Frame " in line:
            print(" ", line)

if __name__ == "__main__":
    # Tick 50: Press FIRE (select English)
    # Tick 70: Release FIRE
    # Tick 100: Press FIRE (select YES for sound)
    # Tick 120: Release FIRE
    # Tick 180: Press FIRE / START (title screen)
    # Tick 200: Release FIRE
    # Tick 240: Press FIRE (New game)
    # Tick 260: Release FIRE
    script = """50:PRESSED:-5
70:RELEASED:-5
100:PRESSED:-5
120:RELEASED:-5
170:PRESSED:-5
190:RELEASED:-5
240:PRESSED:-5
260:RELEASED:-5
300:PRESSED:-5
320:RELEASED:-5
"""
    run_scenario("cv_test1", script, ticks=360)
