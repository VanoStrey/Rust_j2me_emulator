import subprocess
import os
import shutil

jar_path = "/Users/foolvan/Desktop/j2me_market/roms/raw/gravity_defied.jar"
cli_path = "./target/release/j2me-cli"

# We know at tick 210-450 the menu is visible.
# Let's test pressing FIRE (-5) at tick 460, tick 480, tick 500, etc.
# In GD: Menu item 1 -> Press FIRE -> Select Difficulty (100cc) -> Press FIRE -> Select Track -> Press FIRE -> Gameplay starts!
# Let's test an input script that presses FIRE at 460, 490, 520, then runs up to tick 700!

input_script = (
    "460:press:FIRE, 465:release:FIRE, "
    "500:press:FIRE, 505:release:FIRE, "
    "540:press:FIRE, 545:release:FIRE"
)

out_dir = "experiments/exp_phase46_stability_blockers/gd_gameplay_test"
if os.path.exists(out_dir):
    shutil.rmtree(out_dir)
os.makedirs(out_dir, exist_ok=True)

cmd = [
    cli_path,
    "--jar", jar_path,
    "--ticks", "700",
    "--dump-dir", out_dir,
    "--input", input_script
]

res = subprocess.run(cmd, capture_output=True, text=True)
print("Exit code:", res.returncode)
print("Stdout tail:\n", "\n".join(res.stdout.splitlines()[-25:]))
