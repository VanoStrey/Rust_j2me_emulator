import json
import os
import subprocess

def main():
    # 1. 13 from fatal 327
    with open("experiments/exp_phase46_stability_blockers/fatal_327_matrix_report.json") as f:
        rep = json.load(f)["results"]
    
    fatal_candidates = []
    for r in rep:
        loc = r.get("old_exception_location") or ""
        cls = r.get("old_exception_class") or ""
        jar = r.get("jar_path") or ""
        if cls == "java/lang/NullPointerException" and (
            "Cheeky" in jar or "StrangeMaze" in jar or "fourthseal" in jar or "fourthseal" in loc or
            "Canvas" in loc or "Display" in loc or "isDoubleBuffered" in loc or "paint" in loc or
            "startApp" in loc or "GameMidlet" in loc
        ):
            fatal_candidates.append(r["jar_path"])

    # 2. 59 from baseline (first 2300 lines of baseline)
    baseline_candidates = []
    with open("experiments/exp_phase46_stability_blockers/full_baseline/smoke_test_progress.jsonl") as f:
        for idx, line in enumerate(f):
            if idx >= 2300: break
            line = line.strip()
            if not line: continue
            try:
                d = json.loads(line)
                if d.get("runtime_status") == "GUEST_EXCEPTION" and d.get("last_exception_class") == "java/lang/NullPointerException":
                    loc = d.get("last_exception_location") or ""
                    jar = d.get("jar_path") or ""
                    if "Cheeky" in jar or "StrangeMaze" in jar or "fourthseal" in jar or \
                       "Canvas" in loc or "Display" in loc or "isDoubleBuffered" in loc or \
                       "paint" in loc or "startApp" in loc or "GameMidlet" in loc or "init" in loc or \
                       "a" in loc: # canvas/display related
                        baseline_candidates.append(jar)
            except:
                pass

    all_72 = []
    seen = set()
    for j in fatal_candidates + baseline_candidates:
        if j not in seen:
            seen.add(j)
            all_72.append(j)
        if len(all_72) == 72:
            break

    # If fewer than 72, add other NPEs from the first 2300 baseline records to reach exactly 72
    if len(all_72) < 72:
        with open("experiments/exp_phase46_stability_blockers/full_baseline/smoke_test_progress.jsonl") as f:
            for idx, line in enumerate(f):
                if idx >= 2300: break
                line = line.strip()
                if not line: continue
                try:
                    d = json.loads(line)
                    if d.get("runtime_status") == "GUEST_EXCEPTION" and d.get("last_exception_class") == "java/lang/NullPointerException":
                        jar = d["jar_path"]
                        if jar not in seen:
                            seen.add(jar)
                            all_72.append(jar)
                            if len(all_72) == 72:
                                break
                except:
                    pass

    print(f"Collected {len(all_72)} configurations for Cluster #1")
    
    os.makedirs("dumps/cluster1_72_results", exist_ok=True)
    results = []

    for idx, jar in enumerate(all_72):
        out_json = f"dumps/cluster1_72_results/res_{idx}.json"
        cmd = [
            "./target/release/j2me-cli",
            "--run-single", jar,
            "--ticks", "300",
            "--output-json", out_json
        ]
        try:
            res = subprocess.run(cmd, capture_output=True, text=True, timeout=6)
            if os.path.exists(out_json):
                with open(out_json) as sf:
                    data = json.load(sf)
                    results.append({
                        "jar": os.path.basename(jar),
                        "status": data.get("runtime_status"),
                        "fatal": data.get("fatal_guest_exception"),
                        "last_exc": data.get("last_exception_class"),
                        "last_loc": data.get("last_exception_location"),
                        "paint_calls": data.get("paint_calls"),
                        "fb_pubs": data.get("framebuffer_publications"),
                        "unique_fb": data.get("unique_framebuffer_hashes"),
                    })
            else:
                results.append({
                    "jar": os.path.basename(jar),
                    "status": "CLI_ERROR",
                    "error": res.stderr[:100]
                })
        except subprocess.TimeoutExpired:
            results.append({
                "jar": os.path.basename(jar),
                "status": "WALL_TIMEOUT",
                "fatal": False,
            })
        print(f"[{idx+1}/{len(all_72)}] {os.path.basename(jar)}: {results[-1].get('status')}")

    out_report = "experiments/exp_phase46_stability_blockers/cluster1_72_verification_report.json"
    with open(out_report, "w") as f:
        json.dump(results, f, indent=2)

    completed = [r for r in results if r.get("status") == "COMPLETED_WINDOW" and not r.get("fatal")]
    timeouts = [r for r in results if r.get("status") == "WALL_TIMEOUT"]
    exceptions = [r for r in results if r.get("status") == "GUEST_EXCEPTION"]

    print(f"\n--- CLUSTER #1 VERIFICATION SUMMARY ---")
    print(f"Total: {len(results)}")
    print(f"COMPLETED_WINDOW: {len(completed)}")
    print(f"WALL_TIMEOUT: {len(timeouts)}")
    print(f"GUEST_EXCEPTION: {len(exceptions)}")

if __name__ == "__main__":
    main()
