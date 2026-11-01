import json
import os
import subprocess
from concurrent.futures import ProcessPoolExecutor, as_completed
from collections import Counter, defaultdict

def run_single(item):
    idx, d, exe, out_dir = item
    jar_path = d['jar_path']
    entry_class = d.get('entry_point') or 'Main'
    
    tmp_json = f"{out_dir}/trace_tmp_{os.getpid()}_{idx}.json"
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
            return idx, d, {"jar_path": jar_path, "runtime_status": "LOAD_ERROR"}
    except subprocess.TimeoutExpired:
        return idx, d, {"jar_path": jar_path, "runtime_status": "WALL_TIMEOUT"}
    except Exception as e:
        return idx, d, {"jar_path": jar_path, "runtime_status": "HOST_PANIC", "last_error": str(e)}

def main():
    prev_file = 'experiments/exp_phase46_stability_blockers/rerun_49/rerun_phase49_results.json'
    with open(prev_file) as f:
        prev_data = json.load(f)

    # 226 GUEST_EXCEPTION
    targets = [item for item in prev_data if item.get('runtime_status') == 'GUEST_EXCEPTION']
    print(f"Total GUEST_EXCEPTION targets to trace: {len(targets)}")

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
                print(f"Trace progress: {len(results)}/{len(items)}")

    results.sort(key=lambda x: x[0])
    parsed_results = [r[2] for r in results]

    with open(f"{out_dir}/trace_226_results.json", "w") as f:
        json.dump(parsed_results, f, indent=2)

    # Analysis
    print("\n================ TELEMETRY DECOMPOSITION ================")
    npe_list = [r for r in parsed_results if r.get('last_exception_class') == 'java/lang/NullPointerException']
    aioobe_list = [r for r in parsed_results if r.get('last_exception_class') == 'java/lang/ArrayIndexOutOfBoundsException']
    other_list = [r for r in parsed_results if r.get('last_exception_class') not in ('java/lang/NullPointerException', 'java/lang/ArrayIndexOutOfBoundsException')]

    print(f"Total parsed: {len(parsed_results)}")
    print(f"NullPointerException: {len(npe_list)}")
    print(f"ArrayIndexOutOfBoundsException: {len(aioobe_list)}")
    print(f"Other exceptions: {len(other_list)}")

    print("\n--- TOP LAST NATIVE BEFORE NPE ---")
    last_native_npe = Counter()
    unhandled_npe = Counter()
    for r in npe_list:
        recents = r.get('recent_native_calls', [])
        unhandled = r.get('unhandled_natives', [])
        for u in unhandled:
            unhandled_npe[u] += 1
        if recents:
            last_native_npe[recents[-1]] += 1
        else:
            last_native_npe["(no native calls)"] += 1

    for call, cnt in last_native_npe.most_common(12):
        print(f"  {cnt:3d} | {call}")

    print("\n--- TOP UNHANDLED NATIVES BEFORE NPE ---")
    for u, cnt in unhandled_npe.most_common(10):
        print(f"  {cnt:3d} | {u}")

    print("\n--- TOP LAST NATIVE BEFORE ArrayIndexOutOfBoundsException ---")
    last_native_aioobe = Counter()
    unhandled_aioobe = Counter()
    for r in aioobe_list:
        recents = r.get('recent_native_calls', [])
        unhandled = r.get('unhandled_natives', [])
        for u in unhandled:
            unhandled_aioobe[u] += 1
        if recents:
            last_native_aioobe[recents[-1]] += 1
        else:
            last_native_aioobe["(no native calls)"] += 1

    for call, cnt in last_native_aioobe.most_common(10):
        print(f"  {cnt:3d} | {call}")

    print("\n--- TOP UNHANDLED NATIVES BEFORE ArrayIndexOutOfBoundsException ---")
    for u, cnt in unhandled_aioobe.most_common(8):
        print(f"  {cnt:3d} | {u}")

if __name__ == '__main__':
    main()
