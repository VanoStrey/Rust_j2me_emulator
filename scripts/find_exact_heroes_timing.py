import subprocess
import os

# We want to test exact tick timings
# Tick 270: Fire to leave Title screen
# Tick 450: Fire to select Ronin
# Tick 550: LEFT to switch popup to YES
# Tick 650: Fire to confirm Ronin
# Tick 800: Fire to select Spirit 0
# Tick 880: RIGHT to move to Spirit 1
# Tick 960: Fire to select Spirit 1 (popup opens)
# Tick 1040: LEFT to switch spirit popup to YES
# Tick 1120: Fire to confirm Spirit (starts game/level 0!)
# Tick 1250: Control frame
# Tick 1300..1400: Hold DOWN
# Tick 1450: Action frame

script = """
270:p:FIRE
290:r:FIRE
450:p:FIRE
470:r:FIRE
550:p:LEFT
570:r:LEFT
650:p:FIRE
670:r:FIRE
800:p:FIRE
820:r:FIRE
880:p:RIGHT
900:r:RIGHT
960:p:FIRE
980:r:FIRE
1040:p:LEFT
1060:r:LEFT
1120:p:FIRE
1140:r:FIRE
1200:p:FIRE
1220:r:FIRE
1280:p:FIRE
1300:r:FIRE
1380:p:DOWN
1460:r:DOWN
"""

os.makedirs("experiments/heroes_lore/timing_dump", exist_ok=True)
with open("experiments/heroes_lore/timing_input.txt", "w") as f:
    f.write(script.strip())

cmd = [
    "./target/release/j2me-cli",
    "--jar", "/Users/foolvan/Desktop/j2me_market/roms/raw/heroes_lore.jar",
    "--jad", "/Users/foolvan/Desktop/Rust_j2me_emulator/experiments/heroes_lore/heroes_lore.jad",
    "--ticks", "1500",
    "--dump-dir", "experiments/heroes_lore/timing_dump",
    "--input", "experiments/heroes_lore/timing_input.txt"
]

res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
print("Finished. Dumping interesting frames...")
for f in ["frame_450.png", "frame_650.png", "frame_800.png", "frame_960.png", "frame_1120.png", "frame_1260.png", "frame_1350.png", "frame_1470.png", "frame_1500.png"]:
    p = os.path.join("experiments/heroes_lore/timing_dump", f)
    if os.path.exists(p):
        print(f"  {f}: {os.path.getsize(p)} bytes")
