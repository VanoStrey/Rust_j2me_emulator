#!/usr/bin/env python3
import json
import os
import subprocess
import sys
import time

CLI_PATH = os.path.abspath("experiments/exp_phase45_exceptions_resources/j2me-cli")
OLD_REPORT = os.path.abspath("experiments/exp_phase42_full_corpus/smoke_test_report.json")
OUT_JSON = os.path.abspath("experiments/exp_phase45_exceptions_resources/reclassified_523.json")
TEMP_SINGLE = os.path.abspath("/tmp/single_reclassify.json")

def main():
    if not os.path.exists(CLI_PATH):
        print(f"Error: {CLI_PATH} not found", file=sys.stderr)
        sys.exit(1)

    print(f"Loading {OLD_REPORT}...")
    with open(OLD_REPORT) as f:
        old_data = json.load(f)

    target_entries = [
        r for r in old_data.get("results", [])
        if r.get("runtime_status") == "GUEST_EXCEPTION"
    ]
    total = len(target_entries)
    print(f"Found {total} entries previously marked GUEST_EXCEPTION")

    reclassified_records = []
    t_start = time.time()

    caught_count = 0
    uncaught_count = 0
    terminated_by_exc = 0
    completed_window_count = 0
    wall_timeout_count = 0
    unknown_count = 0

    for idx, entry in enumerate(target_entries):
        jar_path = entry.get("jar_path")
        entry_point = entry.get("entry_point")
        launch_id = entry.get("launch_configuration_id", entry.get("file_id"))
        jar_sha = entry.get("jar_sha256")

        if os.path.exists(TEMP_SINGLE):
            os.remove(TEMP_SINGLE)

        cmd = [
            CLI_PATH,
            "--run-single", jar_path,
            "--ticks", "300",
            "--output-json", TEMP_SINGLE
        ]
        if entry_point and entry_point != "default":
            cmd.extend(["--class", entry_point])

        try:
            # 6 second watchdog per entry
            res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=6.0)
            if os.path.exists(TEMP_SINGLE):
                with open(TEMP_SINGLE) as sf:
                    run_data = json.load(sf)
            else:
                run_data = None
        except subprocess.TimeoutExpired:
            run_data = {"runtime_status": "WALL_TIMEOUT", "exit_reason": "TIMEOUT"}
        except Exception as e:
            run_data = {"runtime_status": "HOST_PANIC", "last_error": str(e)}

        if run_data is None:
            new_status = "HOST_PANIC"
            reason = "Process crashed before producing JSON"
            rec = {
                "launch_configuration_id": launch_id,
                "jar_path": jar_path,
                "jar_sha256": jar_sha,
                "old_runtime_status": "GUEST_EXCEPTION",
                "new_runtime_status": new_status,
                "reclassification_reason": reason,
                "guest_exceptions_thrown": 0,
                "guest_exceptions_caught": 0,
                "guest_exceptions_uncaught": 0,
                "fatal_guest_exception": False,
                "uncaught_threads": [],
                "exception_class": None,
                "exception_message": None,
                "exception_location": None,
                "framebuffer_publications": 0,
                "unique_framebuffer_hashes": 0,
            }
        else:
            new_status = run_data.get("runtime_status", "UNKNOWN")
            fatal = run_data.get("fatal_guest_exception", False)
            thrown = run_data.get("guest_exceptions_thrown", 0)
            caught = run_data.get("guest_exceptions_caught", 0)
            uncaught = run_data.get("guest_exceptions_uncaught", 0)
            uncaught_threads = run_data.get("uncaught_exception_threads", [])
            exc_class = run_data.get("last_exception_class")
            exc_msg = run_data.get("last_exception_message")
            exc_loc = run_data.get("last_exception_location")
            fb_pubs = run_data.get("framebuffer_publications", 0)
            fb_unique = run_data.get("unique_framebuffer_hashes", 0)

            if fatal:
                reason = f"Fatal uncaught exception {exc_class} in {uncaught_threads}"
                terminated_by_exc += 1
            elif caught > 0 and uncaught == 0:
                reason = f"Handled exception (caught {caught}/{thrown}) - normal execution continued"
            elif new_status == "COMPLETED_WINDOW":
                reason = "Execution completed allotted virtual window without fatal crash"
            elif new_status == "WALL_TIMEOUT":
                reason = "Wall timeout exceeded in non-yielding loop"
            else:
                reason = f"Reclassified based on telemetry: status={new_status}"

            if caught > 0:
                caught_count += 1
            if uncaught > 0:
                uncaught_count += 1
            if new_status == "COMPLETED_WINDOW":
                completed_window_count += 1
            elif new_status == "WALL_TIMEOUT":
                wall_timeout_count += 1
            elif new_status == "UNKNOWN":
                unknown_count += 1

            rec = {
                "launch_configuration_id": launch_id,
                "jar_path": jar_path,
                "jar_sha256": jar_sha,
                "old_runtime_status": "GUEST_EXCEPTION",
                "new_runtime_status": new_status,
                "reclassification_reason": reason,
                "guest_exceptions_thrown": thrown,
                "guest_exceptions_caught": caught,
                "guest_exceptions_uncaught": uncaught,
                "fatal_guest_exception": fatal,
                "uncaught_threads": uncaught_threads,
                "exception_class": exc_class,
                "exception_message": exc_msg,
                "exception_location": exc_loc,
                "framebuffer_publications": fb_pubs,
                "unique_framebuffer_hashes": fb_unique,
            }

        reclassified_records.append(rec)
        if (idx + 1) % 50 == 0 or idx + 1 == total:
            elapsed = time.time() - t_start
            print(f"[{idx + 1}/{total}] {elapsed:.1f}s - {new_status} ({rec['reclassification_reason'][:40]})")

    out_summary = {
        "total_evaluated": total,
        "caught_exceptions_runs": caught_count,
        "uncaught_exceptions_runs": uncaught_count,
        "fatal_uncaught_terminations": terminated_by_exc,
        "reclassified_completed_window": completed_window_count,
        "reclassified_wall_timeout": wall_timeout_count,
        "reclassified_unknown": unknown_count,
        "records": reclassified_records,
    }

    with open(OUT_JSON, "w") as f:
        json.dump(out_summary, f, indent=2)

    print(f"\nDone! Reclassified records saved to {OUT_JSON}")
    print(f"Total: {total}")
    print(f"Runs with caught exceptions: {caught_count}")
    print(f"Runs with uncaught exceptions: {uncaught_count}")
    print(f"Terminated due to fatal uncaught exception: {terminated_by_exc}")
    print(f"COMPLETED_WINDOW: {completed_window_count}")
    print(f"WALL_TIMEOUT: {wall_timeout_count}")
    print(f"UNKNOWN: {unknown_count}")

if __name__ == "__main__":
    main()
