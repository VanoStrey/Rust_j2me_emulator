import subprocess
import os

dump_dir = "experiments/heroes_lore/exact_dump"
os.makedirs(dump_dir, exist_ok=True)

# Sequence:
# 30: press 53 (skip splash / title)
# 90: press 53 (skip splash)
# 150: press 53 (Title -> Char creation)
# 200: press 53 (Char creation -> Ronin popup)
# 240: press -3 (Ronin popup: NO -> YES)
# 270: press 53 (Ronin popup: Confirm YES -> Spirit selection)
# 400: press 53 (Spirit selection: Select Spirit 0)
# 450: press -4 (Spirit selection: Move cursor to Spirit 1)
# 500: press 53 (Spirit selection: Select Spirit 1 -> Count=2! Confirmation dialog opens!)
# 560: press -3 (Confirmation dialog: NO -> YES!)
# 620: press 53 (Confirmation dialog: Confirm YES! Calls bk.d() -> n.a() -> Game starts!)
# 700..1200: press 53 periodically to advance initial story/dialogue if needed
# 1300..1500: action movement

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
    # Story advance / gameplay ticks
    "760:p:53", "770:r:53",
    "840:p:53", "850:r:53",
    "920:p:53", "930:r:53",
    "1000:p:53", "1010:r:53",
    "1080:p:53", "1090:r:53",
    "1160:p:53", "1170:r:53",
    "1240:p:53", "1250:r:53",
    "1320:p:53", "1330:r:53",
    # Action DOWN
    "1400:p:56", "1480:r:56",
]

script_path = "experiments/heroes_lore/exact_script.txt"
with open(script_path, "w") as f:
    f.write("\n".join(script_lines) + "\n")

cmd = [
    "./target/release/j2me-cli",
    "--jar", "/Users/foolvan/Desktop/j2me_market/roms/raw/heroes_lore.jar",
    "--jad", "experiments/heroes_lore/heroes_lore.jad",
    "--ticks", "1550",
    "--dump-dir", dump_dir,
    "--input", script_path
]

print("Running exact script...")
res = subprocess.run(cmd, capture_output=True, text=True)
print("Exit:", res.returncode)
for l in res.stdout.splitlines():
    if "Frame " in l or "STDOUT" in l:
        print(l)
