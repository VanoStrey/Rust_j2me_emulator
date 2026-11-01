import json
import subprocess
import os
import time

with open("experiments/exp_phase45_exceptions_resources/reclassified_523.json") as f:
    d = json.load(f)

fatal_records = [r for r in d["records"] if r["fatal_guest_exception"]]
print(f"Total fatal records to evaluate: {len(fatal_records)}")

results = []
progress_file = "experiments/exp_phase46_stability_blockers/fatal_327_progress.jsonl"
os.makedirs(os.path.dirname(progress_file), exist_ok=True)

# Load existing progress if any
completed_keys = set()
if os.path.exists(progress_file):
    with open(progress_file) as pf:
        for line in pf:
            if line.strip():
                try:
                    obj = json.loads(line)
                    completed_keys.add(obj["jar_path"])
                    results.append(obj)
                except Exception:
                    pass

print(f"Resuming from {len(completed_keys)} completed runs...")

for idx, r in enumerate(fatal_records):
    jar_path = r["jar_path"]
    if jar_path in completed_keys:
        continue

    tmp_out = f"/tmp/fatal_eval_{idx}.json"
    cmd = [
        "./target/release/j2me-cli",
        "--run-single",
        jar_path,
        "--ticks",
        "300",
        "--output-json",
        tmp_out,
    ]
    t0 = time.time()
    try:
        sub = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        wall_ms = (time.time() - t0) * 1000.0
        if os.path.exists(tmp_out):
            with open(tmp_out) as tf:
                res_data = json.load(tf)
            item = {
                "jar_path": jar_path,
                "jar_sha256": r.get("jar_sha256"),
                "old_status": "GUEST_EXCEPTION",
                "old_exception_class": r.get("exception_class"),
                "old_exception_location": r.get("exception_location"),
                "new_status": res_data.get("runtime_status"),
                "new_exit_reason": res_data.get("exit_reason"),
                "new_exception_class": res_data.get("last_exception_class"),
                "new_exception_location": res_data.get("last_exception_location"),
                "fatal_guest_exception": res_data.get("fatal_guest_exception"),
                "framebuffer_publications": res_data.get("framebuffer_publications", 0),
                "paint_calls": res_data.get("paint_calls", 0),
                "unique_hashes": res_data.get("unique_framebuffer_hashes", 0),
                "total_wall_ms": wall_ms,
            }
        else:
            item = {
                "jar_path": jar_path,
                "jar_sha256": r.get("jar_sha256"),
                "old_status": "GUEST_EXCEPTION",
                "old_exception_class": r.get("exception_class"),
                "old_exception_location": r.get("exception_location"),
                "new_status": "HOST_ERROR",
                "total_wall_ms": wall_ms,
                "error": sub.stderr[:200],
            }
    except subprocess.TimeoutExpired:
        item = {
            "jar_path": jar_path,
            "jar_sha256": r.get("jar_sha256"),
            "old_status": "GUEST_EXCEPTION",
            "old_exception_class": r.get("exception_class"),
            "old_exception_location": r.get("exception_location"),
            "new_status": "WALL_TIMEOUT",
            "total_wall_ms": 10000.0,
        }

    results.append(item)
    completed_keys.add(jar_path)
    with open(progress_file, "a") as pf:
        pf.write(json.dumps(item) + "\n")

    if (idx + 1) % 25 == 0 or idx == len(fatal_records) - 1:
        print(f"[{idx+1}/{len(fatal_records)}] Processed {os.path.basename(jar_path)}: {item.get('new_status')}")

# Save final report
report_path = "experiments/exp_phase46_stability_blockers/fatal_327_matrix_report.json"
with open(report_path, "w") as rf:
    json.dump({"total": len(results), "results": results}, rf, indent=2)
print(f"Saved {report_path}")
