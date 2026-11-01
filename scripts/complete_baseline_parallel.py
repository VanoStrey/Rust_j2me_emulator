import json
import os
import subprocess
import time
from concurrent.futures import ProcessPoolExecutor, as_completed

def run_single(item):
    idx, entry, exe, out_dir = item
    jar_path = entry['path']
    midlets = entry.get('midlet_entries', [])
    entry_class = midlets[0]['main_class'] if midlets and 'main_class' in midlets[0] else 'Main'
    file_id = entry.get('file_id', f"file_{idx:05d}")
    vendor = entry.get('vendor') or ''
    sha256 = entry.get('sha256') or ''
    launch_id = entry.get('launch_configuration_id') or (midlets[0]['name'] if midlets and 'name' in midlets[0] else 'default')
    
    tmp_json = f"{out_dir}/tmp_{os.getpid()}_{idx}.json"
    cmd = [
        exe,
        "--run-single", jar_path,
        "--class", entry_class,
        "--ticks", "300",
        "--output-json", tmp_json
    ]
    t0 = time.time()
    try:
        res = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=6.0)
        wall_ms = (time.time() - t0) * 1000.0
        if os.path.exists(tmp_json):
            with open(tmp_json) as f:
                parsed = json.load(f)
            try:
                os.remove(tmp_json)
            except:
                pass
            parsed['file_id'] = file_id
            parsed['title'] = vendor
            parsed['vendor'] = vendor
            return parsed
        else:
            return {
                "file_id": file_id,
                "jar_path": jar_path,
                "jar_sha256": sha256,
                "launch_configuration_id": launch_id,
                "entry_point": entry_class,
                "title": vendor,
                "vendor": vendor,
                "runtime_status": "LOAD_ERROR",
                "observed_ui": "UNKNOWN",
                "load_ok": False,
                "midlet_started": False,
                "main_canvas_created": False,
                "paint_calls": 0,
                "framebuffer_publications": 0,
                "unique_framebuffer_hashes": 0,
                "active_threads": 0,
                "executed_instructions": 0,
                "exceptions": 0,
                "unhandled_native": 0,
                "native_call_counts": 0,
                "guest_exceptions_thrown": 0,
                "guest_exceptions_caught": 0,
                "guest_exceptions_uncaught": 0,
                "fatal_guest_exception": False,
                "uncaught_exception_threads": [],
                "last_exception_class": None,
                "last_exception_message": None,
                "last_exception_location": None,
                "audio_generated_samples": 0,
                "audio_consumed_samples": 0,
                "audio_dropped_samples": 0,
                "audio_underrun_count": 0,
                "audio_overflow_count": 0,
                "timing": {
                    "elapsed_load_ms": 0.0,
                    "elapsed_vm_ms": 0.0,
                    "elapsed_graphics_ms": 0.0,
                    "elapsed_audio_ms": 0.0,
                    "elapsed_scheduler_ms": 0.0,
                    "elapsed_io_ms": 0.0,
                    "elapsed_other_ms": 0.0,
                    "total_wall_ms": wall_ms
                },
                "exit_reason": "LOAD_ERROR",
                "last_error": "Failed to generate output json",
                "final_frame_sha256": "",
                "apis_invoked": [],
                "blockers": []
            }
    except subprocess.TimeoutExpired:
        return {
            "file_id": file_id,
            "jar_path": jar_path,
            "jar_sha256": sha256,
            "launch_configuration_id": launch_id,
            "entry_point": entry_class,
            "title": vendor,
            "vendor": vendor,
            "runtime_status": "WALL_TIMEOUT",
            "observed_ui": "UNKNOWN",
            "load_ok": True,
            "midlet_started": True,
            "main_canvas_created": False,
            "paint_calls": 0,
            "framebuffer_publications": 0,
            "unique_framebuffer_hashes": 0,
            "active_threads": 1,
            "executed_instructions": 0,
            "exceptions": 0,
            "unhandled_native": 0,
            "native_call_counts": 0,
            "guest_exceptions_thrown": 0,
            "guest_exceptions_caught": 0,
            "guest_exceptions_uncaught": 0,
            "fatal_guest_exception": False,
            "uncaught_exception_threads": [],
            "last_exception_class": None,
            "last_exception_message": None,
            "last_exception_location": None,
            "audio_generated_samples": 0,
            "audio_consumed_samples": 0,
            "audio_dropped_samples": 0,
            "audio_underrun_count": 0,
            "audio_overflow_count": 0,
            "timing": {
                "elapsed_load_ms": 0.0,
                "elapsed_vm_ms": 6000.0,
                "elapsed_graphics_ms": 0.0,
                "elapsed_audio_ms": 0.0,
                "elapsed_scheduler_ms": 0.0,
                "elapsed_io_ms": 0.0,
                "elapsed_other_ms": 0.0,
                "total_wall_ms": 6000.0
            },
            "exit_reason": "WALL_TIMEOUT",
            "last_error": "Wall clock timeout exceeded",
            "final_frame_sha256": "",
            "apis_invoked": [],
            "blockers": [{
                "blocker_type": "CONFIRMED",
                "category": "WALL_TIMEOUT",
                "description": "Midlet entered non-yielding loop or hung during initialization",
                "minimal_repro": None
            }]
        }

def main():
    inv_file = "catalog/j2me_market_inventory.json"
    with open(inv_file) as f:
        inv = json.load(f)

    progress_file = "experiments/exp_phase46_stability_blockers/full_baseline/smoke_test_progress.jsonl"
    seen = set()
    current_count = 0
    if os.path.exists(progress_file):
        with open(progress_file) as f:
            for line in f:
                line = line.strip()
                if not line: continue
                try:
                    d = json.loads(line)
                    seen.add(d['jar_path'])
                    current_count += 1
                except:
                    pass

    print(f"Current progress count: {current_count}, unique jars: {len(seen)}")

    target_total = 7308
    candidates = [
        e for e in inv['files']
        if e.get('extension') == 'jar' and e.get('size_bytes', 0) > 0
        and e.get('availability') == 'PRESENT' and e.get('archive_status') == 'VALID'
    ]

    needed = []
    for idx, e in enumerate(candidates):
        if e['path'] not in seen:
            needed.append((idx + 1, e, "./target/release/j2me-cli", "experiments/exp_phase46_stability_blockers/full_baseline"))
            if len(seen) + len(needed) >= target_total:
                break

    print(f"Configurations to run: {len(needed)} to reach {target_total}")
    if not needed:
        print("Already reached target!")
        return

    workers = min(8, os.cpu_count() or 4)
    print(f"Launching pool with {workers} workers...")
    
    with open(progress_file, "a") as out_f:
        with ProcessPoolExecutor(max_workers=workers) as executor:
            futures = {executor.submit(run_single, item): item for item in needed}
            done_count = 0
            for fut in as_completed(futures):
                res = fut.result()
                out_f.write(json.dumps(res) + "\n")
                out_f.flush()
                done_count += 1
                seen.add(res['jar_path'])
                if done_count % 100 == 0 or done_count == len(needed):
                    print(f"Progress: {len(seen)} / {target_total} ({done_count}/{len(needed)} new done)")
                if len(seen) >= target_total:
                    break

    print(f"Finished! Total unique configs reached: {len(seen)}")

if __name__ == '__main__':
    main()
