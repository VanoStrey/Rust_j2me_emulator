import subprocess
import os

def test_keys(keys, name):
    script_path = f"/tmp/{name}_input.txt"
    lines = []
    tick = 70
    for k in keys:
        lines.append(f"{tick}:PRESSED:{k}")
        lines.append(f"{tick+15}:RELEASED:{k}")
        tick += 40
    with open(script_path, "w") as f:
        f.write("\n".join(lines) + "\n")
        
    out_dir = f"dumps/{name}_frames"
    os.makedirs(out_dir, exist_ok=True)
    
    cmd = [
        "./target/release/j2me-cli",
        "--jar", "/Users/foolvan/Desktop/j2me_market/roms/Games/CastlevaniaAoS.jar",
        "--ticks", str(tick + 60),
        "--dump-dir", out_dir,
        "--input", script_path
    ]
    res = subprocess.run(cmd, capture_output=True, text=True)
    for line in res.stdout.splitlines():
        if "Frame " in line:
            print(" ", line)

if __name__ == "__main__":
    print("Testing SoftLeft (-6)...")
    test_keys([-6, -6, -6, -6], "cv_softleft")
