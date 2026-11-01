import json
import os
import subprocess
from concurrent.futures import ProcessPoolExecutor, as_completed
from collections import Counter

def run_single(item):
    idx, d, exe, out_dir = item
    jar_path = d['jar_path']
    entry_class = d.get('entry_point') or 'Main'
    
    tmp_json = f"{out_dir}/p51_tmp_{os.getpid()}_{idx}.json"
    cmd = [
        exe,
        "--run-single", jar_path,
        "--class", entry_class,
        "--ticks", "300",
        "--output-json", tmp_json
    ]
    try:
        subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=7.0)
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
    prev_file = 'experiments/exp_phase46_stability_blockers/rerun_50/phase50_204_results.json'
    with open(prev_file) as f:
        prev_data = json.load(f)

    # 191 targets that were GUEST_EXCEPTION
    targets = [item for item in prev_data if item.get('new_status') == 'GUEST_EXCEPTION']
    print(f"Total targets to test in Phase 5.1: {len(targets)}")

    out_dir = "experiments/exp_phase46_stability_blockers/rerun_51"
    os.makedirs(out_dir, exist_ok=True)
    exe = "./target/release/j2me-cli"

    items = [(idx, d, exe, out_dir) for idx, d in enumerate(targets)]
    results = []
    workers = min(8, os.cpu_count() or 4)
    with ProcessPoolExecutor(max_workers=workers) as executor:
        futures = {executor.submit(run_single, it): it for it in items}
        for fut in as_completed(futures):
            res = fut.result()
            results.append(res)
            if len(results) % 40 == 0 or len(results) == len(items):
                print(f"Phase 5.1 rerun progress: {len(results)}/{len(items)}")

    # Sort results by original idx
    results.sort(key=lambda x: x[0])

    transitions = Counter()
    status_counts = Counter()
    output_records = []

    for idx, old_entry, new_result in results:
        old_status = old_entry.get('new_status')
        new_status = new_result.get('runtime_status', 'UNKNOWN')
        trans_key = f"{old_status} -> {new_status}"
        transitions[trans_key] += 1
        status_counts[new_status] += 1
        output_records.append({
            "jar_path": old_entry['jar_path'],
            "entry_point": old_entry.get('entry_point'),
            "old_status": old_status,
            "new_status": new_status,
            "instructions": new_result.get("instructions_executed", 0),
            "paint_calls": new_result.get("paint_calls", 0),
            "last_exception_class": new_result.get("last_exception_class"),
            "recent_native_calls": new_result.get("recent_native_calls", []),
            "unhandled_natives": new_result.get("unhandled_natives", [])
        })

    out_file = f"{out_dir}/phase51_191_results.json"
    with open(out_file, "w") as f:
        json.dump(output_records, f, indent=2)

    print("\n=== PHASE 5.1 TRANSITION RESULTS (191 CONFIGURATIONS) ===")
    for trans, count in transitions.most_common():
        pct = (count / len(targets)) * 100
        print(f"  {trans}: {count} ({pct:.1f}%)")

    print("\n=== PHASE 5.1 NEW STATUS TOTALS ===")
    for st, count in status_counts.most_common():
        pct = (count / len(targets)) * 100
        print(f"  {st}: {count} ({pct:.1f}%)")

    guest_exc = status_counts.get('GUEST_EXCEPTION', 0)
    print(f"\nFinal remaining GUEST_EXCEPTION: {guest_exc} (was {len(targets)})")

if __name__ == '__main__':
    main()
