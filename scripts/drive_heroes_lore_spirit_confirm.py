import subprocess
import os

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
1280:p:FIRE
1300:r:FIRE
1380:p:RIGHT
1400:r:RIGHT
1460:p:FIRE
1480:r:FIRE
1540:p:LEFT
1560:r:LEFT
1620:p:FIRE
1640:r:FIRE
1720:p:FIRE
1740:r:FIRE
1820:p:FIRE
1840:r:FIRE
1920:p:FIRE
1940:r:FIRE
2020:p:DOWN
2120:r:DOWN
"""

os.makedirs("experiments/heroes_lore/spirit_dump", exist_ok=True)
with open("experiments/heroes_lore/spirit_input.txt", "w") as f:
    f.write(script.strip())

cmd = [
    "./target/release/j2me-cli",
    "--jar", "/Users/foolvan/Desktop/j2me_market/roms/raw/heroes_lore.jar",
    "--jad", "/Users/foolvan/Desktop/Rust_j2me_emulator/experiments/heroes_lore/heroes_lore.jad",
    "--ticks", "2200",
    "--dump-dir", "experiments/heroes_lore/spirit_dump",
    "--input", "experiments/heroes_lore/spirit_input.txt"
]

res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
print("Finished. Dumping frames past 1500...")
frames = sorted([f for f in os.listdir("experiments/heroes_lore/spirit_dump") if f.endswith(".png")])
for f in frames:
    tick = int(f.split("_")[1].split(".")[0])
    if tick >= 1400:
        p = os.path.join("experiments/heroes_lore/spirit_dump", f)
        print(f"  {f}: {os.path.getsize(p)} bytes")
