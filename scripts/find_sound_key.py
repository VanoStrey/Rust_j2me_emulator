import subprocess
import os

keys_to_test = [
    ("softleft", -6),
    ("fire", -5),
    ("key5", 53),
    ("softright", -7),
    ("key1", 49),
    ("key0", 48),
]

for name, k in keys_to_test:
    script_path = f"/tmp/cv_test_{name}.txt"
    with open(script_path, "w") as f:
        # At tick 60: press SoftLeft (-6) to select English
        f.write("60:PRESSED:-6\n")
        f.write("75:RELEASED:-6\n")
        # At tick 140: press the key to test
        f.write(f"140:PRESSED:{k}\n")
        f.write(f"160:RELEASED:{k}\n")
        
    out_dir = f"dumps/cv_{name}_frames"
    os.makedirs(out_dir, exist_ok=True)
    
    cmd = [
        "./target/release/j2me-cli",
        "--jar", "/Users/foolvan/Desktop/j2me_market/roms/Games/CastlevaniaAoS.jar",
        "--ticks", "200",
        "--dump-dir", out_dir,
        "--input", script_path
    ]
    res = subprocess.run(cmd, capture_output=True, text=True)
    last_frame = ""
    for line in res.stdout.splitlines():
        if "Frame 180" in line or "Frame 200" in line:
            last_frame = line
    print(f"Key {name} ({k}): {last_frame}")
