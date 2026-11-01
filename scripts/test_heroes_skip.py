import subprocess
import os

dump_dir = "experiments/heroes_lore/skip_dump"
os.makedirs(dump_dir, exist_ok=True)

# Dialogue starts around tick 1000.
# At 1040: press SOFT_RIGHT (-7) to SKIP dialogue!
# At 1100: dialogue is skipped, hero standing on map!
# Control frame: tick 1200
# Action: press DOWN (56) from tick 1220 to 1340
# Action frame: tick 1360

script_lines = [
    "30:p:53", "40:r:53",
    "90:p:53", "100:r:53",
    "150:p:53", "160:r:53",
    "200:p:53", "210:r:53",
    "240:p:-3", "250:r:-3",
    "270:p:53", "280:r:53",
    # At 390, "GUARDIANS SELECTION." dismiss with FIRE
    "400:p:53", "410:r:53",
    # At 450: select Spirit 0 (Hestia) -> count = 1
    "450:p:53", "460:r:53",
    # At 500: move cursor RIGHT to Spirit 1 (Ran)
    "500:p:-4", "510:r:-4",
    # At 550: select Spirit 1 (Ran) -> count = 2 -> "ARE YOU SURE? YES NO" dialog opens!
    "550:p:53", "560:r:53",
    # At 620: press LEFT (-3 or 52) to switch seal from NO to YES
    "620:p:52", "630:r:52",
    # At 680: press FIRE (53) to confirm YES and launch game via bk.d()!
    "680:p:53", "690:r:53",
    # At tick 1040: press SOFT_RIGHT (-7) to skip dialogue!
    "1040:p:-7", "1050:r:-7",
    "1080:p:-7", "1090:r:-7",
    "1120:p:-7", "1130:r:-7",
    # Action DOWN from tick 1250 to 1380
    "1250:p:56", "1380:r:56",
]

script_path = "experiments/heroes_lore/skip_script.txt"
with open(script_path, "w") as f:
    f.write("\n".join(script_lines) + "\n")

cmd = [
    "./target/release/j2me-cli",
    "--jar", "/Users/foolvan/Desktop/j2me_market/roms/raw/heroes_lore.jar",
    "--jad", "experiments/heroes_lore/heroes_lore.jad",
    "--ticks", "1450",
    "--dump-dir", dump_dir,
    "--input", script_path
]

print("Running skip script...")
res = subprocess.run(cmd, capture_output=True, text=True)
print("Exit:", res.returncode)
for l in res.stdout.splitlines():
    if "Frame " in l or "STDOUT" in l:
        print(l)
