import json
import os
import subprocess
import sys

def main():
    jars = set()
    
    # 1. Check fatal_327_progress.jsonl
    p1 = "experiments/exp_phase46_stability_blockers/fatal_327_progress.jsonl"
    if os.path.exists(p1):
        with open(p1) as f:
            for line in f:
                line = line.strip()
                if not line: continue
                try:
                    d = json.loads(line)
                    if d.get("new_exception_class") == "java/lang/IllegalMonitorStateException" or \
                       d.get("last_exception_class") == "java/lang/IllegalMonitorStateException":
                        jars.add(d["jar_path"])
                except:
                    pass

    # 2. Check full baseline
    p2 = "experiments/exp_phase46_stability_blockers/full_baseline/smoke_test_progress.jsonl"
    if os.path.exists(p2):
        with open(p2) as f:
            for line in f:
                line = line.strip()
                if not line: continue
                try:
                    d = json.loads(line)
                    if d.get("last_exception_class") == "java/lang/IllegalMonitorStateException" or \
                       "IllegalMonitorStateException" in str(d.get("last_error", "")):
                        jars.add(d["jar_path"])
                except:
                    pass

    print(f"Found {len(jars)} JARs with IllegalMonitorStateException")
    jar_list = sorted(list(jars))
    
    results = []
    os.makedirs("dumps/illegal_monitor_cluster", exist_ok=True)
    
    for idx, jar in enumerate(jar_list):
        out_json = f"dumps/illegal_monitor_cluster/res_{idx}.json"
        
        # If already done and valid, load it
        if os.path.exists(out_json) and os.path.getsize(out_json) > 10:
            try:
                with open(out_json) as f:
                    data = json.loads(f.read())
                    results.append({
                        "jar": os.path.basename(jar),
                        "status": data.get("runtime_status"),
                        "fatal": data.get("fatal_guest_exception"),
                        "uncaught": data.get("guest_exceptions_uncaught"),
                        "last_exc": data.get("last_exception_class"),
                        "last_loc": data.get("last_exception_location"),
                        "paint_calls": data.get("paint_calls"),
                        "fb_pubs": data.get("framebuffer_publications"),
                        "unique_fb": data.get("unique_framebuffer_hashes"),
                    })
                    print(f"[{idx+1}/{len(jar_list)}] {os.path.basename(jar)}: (cached) {data.get('runtime_status')}")
                    continue
            except Exception:
                pass

        cmd = [
            "./target/release/j2me-cli",
            "--run-single", jar,
            "--ticks", "300",
            "--output-json", out_json
        ]
        try:
            res = subprocess.run(cmd, capture_output=True, text=True, timeout=6)
            if os.path.exists(out_json):
                with open(out_json) as f:
                    data = json.loads(f.read())
                    status = data.get("runtime_status")
                    results.append({
                        "jar": os.path.basename(jar),
                        "status": status,
                        "fatal": data.get("fatal_guest_exception"),
                        "uncaught": data.get("guest_exceptions_uncaught"),
                        "last_exc": data.get("last_exception_class"),
                        "last_loc": data.get("last_exception_location"),
                        "paint_calls": data.get("paint_calls"),
                        "fb_pubs": data.get("framebuffer_publications"),
                        "unique_fb": data.get("unique_framebuffer_hashes"),
                    })
                    print(f"[{idx+1}/{len(jar_list)}] {os.path.basename(jar)}: {status}")
            else:
                results.append({
                    "jar": os.path.basename(jar),
                    "status": "CLI_ERROR",
                    "error": res.stderr[:200]
                })
                print(f"[{idx+1}/{len(jar_list)}] {os.path.basename(jar)}: CLI_ERROR")
        except subprocess.TimeoutExpired:
            results.append({
                "jar": os.path.basename(jar),
                "status": "WALL_TIMEOUT",
                "fatal": False,
                "uncaught": 0,
            })
            print(f"[{idx+1}/{len(jar_list)}] {os.path.basename(jar)}: WALL_TIMEOUT (6s)")

    # Save summary report
    report_file = "experiments/exp_phase46_stability_blockers/illegal_monitor_transition_matrix.json"
    with open(report_file, "w") as f:
        json.dump(results, f, indent=2)

    # Summary
    completed = [r for r in results if r.get("status") == "COMPLETED_WINDOW" and not r.get("fatal")]
    timeouts = [r for r in results if r.get("status") == "WALL_TIMEOUT"]
    still_exc = [r for r in results if r.get("status") == "GUEST_EXCEPTION"]
    still_illegal_monitor = [r for r in results if r.get("last_exc") == "java/lang/IllegalMonitorStateException"]
    
    print(f"\n============================================================")
    print(f"   ILLEGAL_MONITOR_STATE_EXCEPTION CLUSTER RESULTS (PHASE 4.8)")
    print(f"============================================================")
    print(f"Total Configurations Tested: {len(results)}")
    print(f"Transitioned to COMPLETED_WINDOW (Clean): {len(completed)}")
    print(f"WALL_TIMEOUT: {len(timeouts)}")
    print(f"Still GUEST_EXCEPTION: {len(still_exc)}")
    print(f"Still IllegalMonitorStateException: {len(still_illegal_monitor)}")
    print(f"Saved full transition report to {report_file}")

if __name__ == "__main__":
    main()
