import json
import os
import subprocess
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from collections import Counter

def run_single(item):
    idx, d, exe, out_dir = item
    jar_path = d['jar_path']
    entry_class = d.get('entry_point') or 'Main'
    
    tmp_json = f"{out_dir}/rerun_tmp_{os.getpid()}_{idx}.json"
    cmd = [
        exe,
        "--run-single", jar_path,
        "--class", entry_class,
        "--ticks", "300",
        "--output-json", tmp_json
    ]
    try:
        subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=6.0)
        if os.path.exists(tmp_json):
            with open(tmp_json) as f:
                parsed = json.load(f)
            try:
                os.remove(tmp_json)
            except:
                pass
            return parsed
        else:
            return {"jar_path": jar_path, "runtime_status": "LOAD_ERROR", "last_exception_class": None}
    except subprocess.TimeoutExpired:
        return {"jar_path": jar_path, "runtime_status": "WALL_TIMEOUT", "last_exception_class": None}

def main():
    p = 'experiments/exp_phase46_stability_blockers/full_baseline/smoke_test_progress.jsonl'
    seen = {}
    with open(p) as f:
        for line in f:
            line = line.strip()
            if not line: continue
            d = json.loads(line)
            seen[d['jar_path']] = d

    exceptions = [d for d in seen.values() if d.get('runtime_status') == 'GUEST_EXCEPTION']
    print(f"Total GUEST_EXCEPTION configs to rerun: {len(exceptions)}")

    out_dir = "experiments/exp_phase46_stability_blockers/rerun_48"
    os.makedirs(out_dir, exist_ok=True)

    items = [(idx, d, "./target/release/j2me-cli", out_dir) for idx, d in enumerate(exceptions)]

    workers = min(8, os.cpu_count() or 4)
    results = []
    with ProcessPoolExecutor(max_workers=workers) as executor:
        futures = {executor.submit(run_single, it): it for it in items}
        for fut in as_completed(futures):
            res = fut.result()
            results.append(res)
            if len(results) % 50 == 0 or len(results) == len(items):
                print(f"Rerun progress: {len(results)}/{len(items)}")

    out_file = f"{out_dir}/delta_rerun_results.json"
    with open(out_file, "w") as f:
        json.dump(results, f, indent=2)

    status_counts = Counter(r.get('runtime_status') for r in results)
    new_completed = status_counts.get('COMPLETED_WINDOW', 0)
    print("\n--- DELTA RERUN RESULTS (Phase 4.8) ---")
    print(f"Total rerun: {len(results)}")
    print(f"Status distribution: {status_counts}")
    print(f"New COMPLETED_WINDOW: +{new_completed}")

    remaining_exceptions = [
        r.get('last_exception_class') or 'Unknown'
        for r in results if r.get('runtime_status') == 'GUEST_EXCEPTION'
    ]
    top3_exc = Counter(remaining_exceptions).most_common(3)
    print(f"Top 3 remaining exceptions: {top3_exc}")

if __name__ == '__main__':
    main()
