import json
import subprocess
import os

with open("/tmp/p0_manifest.json") as f:
    roms = json.load(f)

results = []
for r in roms:
    path = r["path"]
    rid = r["id"]
    jar_name = r["jar"]
    out_json = f"/tmp/p0_{rid}.json"
    cmd = ["./target/release/j2me-cli", "--run-single", path, "--ticks", "300", "--output-json", out_json]
    res = subprocess.run(cmd, capture_output=True, text=True)
    if os.path.exists(out_json):
        with open(out_json) as jf:
            data = json.load(jf)
        item = {
            "id": rid,
            "jar": jar_name,
            "primary_outcome": data["runtime_status"],
            "exit_reason": data["exit_reason"],
            "framebuffer_publications": data["framebuffer_publications"],
            "paint_calls": data["paint_calls"],
            "unique_hashes": data["unique_framebuffer_hashes"],
            "last_error": data.get("last_error"),
            "fatal_guest_exception": data.get("fatal_guest_exception"),
            "total_wall_ms": data["timing"]["total_wall_ms"]
        }
    else:
        item = {
            "id": rid,
            "jar": jar_name,
            "primary_outcome": "HOST_ERROR",
            "last_error": res.stderr[:200]
        }
    outcome = item.get("primary_outcome")
    fb = item.get("framebuffer_publications")
    err = item.get("last_error")
    print(f"[{rid}] {jar_name}: outcome={outcome} fb={fb} err={err}")
    results.append(item)

with open("experiments/exp_phase46_stability_blockers/p0_12_results_300ticks.json", "w") as out:
    json.dump(results, out, indent=2)
print("Saved experiments/exp_phase46_stability_blockers/p0_12_results_300ticks.json")
