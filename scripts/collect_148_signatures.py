import json
import os
import subprocess
from concurrent.futures import ProcessPoolExecutor, as_completed
from collections import Counter

def run_single(item):
    idx, d, exe = item
    jar_path = d['jar_path']
    entry_class = d.get('entry_point') or 'Main'
    
    tmp_json = f"/tmp/p55_diag_{os.getpid()}_{idx}.json"
    cmd = [
        exe,
        "--run-single", jar_path,
        "--class", entry_class,
        "--ticks", "300",
        "--output-json", tmp_json
    ]
    try:
        subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=8.0)
        if os.path.exists(tmp_json):
            with open(tmp_json) as f:
                parsed = json.load(f)
            try:
                os.remove(tmp_json)
            except:
                pass
            return idx, d, parsed
        else:
            return idx, d, {"jar_path": jar_path, "runtime_status": "LOAD_ERROR"}
    except subprocess.TimeoutExpired:
        return idx, d, {"jar_path": jar_path, "runtime_status": "WALL_TIMEOUT"}
    except Exception as e:
        return idx, d, {"jar_path": jar_path, "runtime_status": "HOST_PANIC", "last_error": str(e)}

def main():
    prev_file = 'experiments/exp_phase46_stability_blockers/rerun_51/phase51_191_results.json'
    with open(prev_file) as f:
        prev_data = json.load(f)

    # Filter the 148 remaining GUEST_EXCEPTION
    targets = [item for item in prev_data if item.get('new_status') == 'GUEST_EXCEPTION']
    print(f"Total GUEST_EXCEPTION targets to analyze: {len(targets)}")

    exe = "./target/release/j2me-cli"
    items = [(idx, d, exe) for idx, d in enumerate(targets)]
    results = []
    workers = min(8, os.cpu_count() or 4)

    with ProcessPoolExecutor(max_workers=workers) as executor:
        futures = {executor.submit(run_single, it): it for it in items}
        for fut in as_completed(futures):
            res = fut.result()
            results.append(res)
            if len(results) % 30 == 0 or len(results) == len(items):
                print(f"Diagnostics progress: {len(results)}/{len(items)}")

    # Sort results
    results.sort(key=lambda x: x[0])

    signatures = Counter()
    exc_classes = Counter()
    locations = Counter()
    details = []

    for idx, d, res in results:
        status = res.get("runtime_status")
        exc_cls = res.get("last_exception_class") or "None"
        exc_msg = res.get("last_exception_message") or ""
        exc_loc = res.get("last_exception_location") or "unknown"
        
        sig = f"[{exc_cls}] {exc_msg} at {exc_loc}"
        if status == "GUEST_EXCEPTION":
            signatures[sig] += 1
            exc_classes[exc_cls] += 1
            locations[exc_loc] += 1
            details.append({
                "jar_path": d["jar_path"],
                "entry_point": d.get("entry_point"),
                "status": status,
                "exception_class": exc_cls,
                "message": exc_msg,
                "location": exc_loc,
                "unhandled_natives": res.get("unhandled_natives", []),
                "recent_calls": res.get("recent_native_calls", [])
            })

    print(f"\n================ EXCEPTION CLASS BREAKDOWN ================")
    for cls_name, cnt in exc_classes.most_common():
        print(f"  {cls_name}: {cnt}")

    print(f"\n================ TOP 20 CRASH SIGNATURES ================")
    for sig, cnt in signatures.most_common(20):
        print(f"  ({cnt}x) {sig}")

    out_file = "experiments/phase55_148_signatures.json"
    with open(out_file, "w") as f:
        json.dump(details, f, indent=2)
    print(f"\nSaved detailed signatures to: {out_file}")

if __name__ == "__main__":
    main()
