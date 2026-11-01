import json
import os
import subprocess
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
            return idx, d, parsed
        else:
            return idx, d, {"jar_path": jar_path, "runtime_status": "LOAD_ERROR", "last_exception_class": None}
    except subprocess.TimeoutExpired:
        return idx, d, {"jar_path": jar_path, "runtime_status": "WALL_TIMEOUT", "last_exception_class": None}
    except Exception as e:
        return idx, d, {"jar_path": jar_path, "runtime_status": "HOST_PANIC", "last_exception_class": str(e)}

def main():
    prev_file = 'experiments/exp_phase46_stability_blockers/rerun_48/delta_rerun_results.json'
    with open(prev_file) as f:
        prev_data = json.load(f)

    # Filter exactly the 240 GUEST_EXCEPTION from delta_rerun_results.json
    targets = [item for item in prev_data if item.get('runtime_status') == 'GUEST_EXCEPTION']
    print(f"Total GUEST_EXCEPTION targets: {len(targets)}")

    out_dir = "experiments/exp_phase46_stability_blockers/rerun_49"
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
                print(f"Progress: {len(results)}/{len(items)}")

    # Sort by idx
    results.sort(key=lambda x: x[0])

    completed_window = 0
    stack_shifted = 0
    same = 0
    wall_timeout = 0
    host_panic = 0

    saved_results = []
    for idx, prev_d, new_d in results:
        saved_results.append(new_d)
        st = new_d.get('runtime_status')
        if st == 'COMPLETED_WINDOW':
            completed_window += 1
        elif st == 'WALL_TIMEOUT':
            wall_timeout += 1
        elif st == 'HOST_PANIC':
            host_panic += 1
        elif st == 'GUEST_EXCEPTION':
            prev_loc = prev_d.get('last_exception_location')
            new_loc = new_d.get('last_exception_location')
            prev_exc = prev_d.get('last_exception_class')
            new_exc = new_d.get('last_exception_class')
            if prev_loc != new_loc or prev_exc != new_exc:
                stack_shifted += 1
            else:
                same += 1
        else:
            stack_shifted += 1

    with open(f"{out_dir}/rerun_phase49_results.json", "w") as f:
        json.dump(saved_results, f, indent=2)

    print("\n--- PHASE 4.9 RERUN 240 MATRIX ---")
    print(f"COMPLETED_WINDOW: {completed_window} (+{completed_window})")
    print(f"STACK_SHIFTED: {stack_shifted}")
    print(f"SAME: {same}")
    print(f"WALL_TIMEOUT: {wall_timeout}")
    print(f"HOST_PANIC: {host_panic}")

if __name__ == '__main__':
    main()
