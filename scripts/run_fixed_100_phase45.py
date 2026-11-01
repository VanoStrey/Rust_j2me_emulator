#!/usr/bin/env python3
import json
import os
import subprocess
import sys
import time

CLI_PATH = os.path.abspath("experiments/exp_phase45_exceptions_resources/j2me-cli")
OLD_100_REPORT = os.path.abspath("experiments/exp_phase44_platform_contracts/sample100/smoke_test_report.json")
OUT_DIR = os.path.abspath("experiments/exp_phase45_exceptions_resources/sample100")
OUT_JSON = os.path.join(OUT_DIR, "smoke_test_report.json")
DIFF_JSON = os.path.join(OUT_DIR, "transitions_diff.json")
TEMP_SINGLE = os.path.abspath("/tmp/single_100_run.json")

def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    with open(OLD_100_REPORT) as f:
        old_data = json.load(f)

    entries = old_data.get("results", [])
    print(f"Loaded {len(entries)} entries from Phase 4.4 sample 100")

    new_results = []
    transitions = {}
    transitions_detail = []

    t_start = time.time()
    for idx, old_entry in enumerate(entries):
        jar_path = old_entry["jar_path"]
        entry_point = old_entry.get("entry_point")
        launch_id = old_entry.get("launch_configuration_id", old_entry.get("file_id"))
        old_status = old_entry["runtime_status"]

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
            res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=6.0)
            if os.path.exists(TEMP_SINGLE):
                with open(TEMP_SINGLE) as sf:
                    run_data = json.load(sf)
            else:
                run_data = None
        except subprocess.TimeoutExpired:
            run_data = {
                "file_id": old_entry.get("file_id"),
                "jar_path": jar_path,
                "jar_sha256": old_entry.get("jar_sha256"),
                "launch_configuration_id": launch_id,
                "runtime_status": "WALL_TIMEOUT",
                "exit_reason": "TIMEOUT",
                "framebuffer_publications": 0,
                "executed_instructions": 0,
                "timing": {"total_wall_ms": 6000.0},
                "fatal_guest_exception": False,
                "guest_exceptions_thrown": 0,
                "guest_exceptions_caught": 0,
                "guest_exceptions_uncaught": 0,
            }
        except Exception as e:
            run_data = {
                "file_id": old_entry.get("file_id"),
                "jar_path": jar_path,
                "jar_sha256": old_entry.get("jar_sha256"),
                "launch_configuration_id": launch_id,
                "runtime_status": "HOST_PANIC",
                "last_error": str(e),
                "framebuffer_publications": 0,
                "executed_instructions": 0,
                "timing": {"total_wall_ms": 0.0},
                "fatal_guest_exception": False,
                "guest_exceptions_thrown": 0,
                "guest_exceptions_caught": 0,
                "guest_exceptions_uncaught": 0,
            }

        if run_data is None:
            new_status = "HOST_PANIC"
            run_data = {
                "file_id": old_entry.get("file_id"),
                "jar_path": jar_path,
                "jar_sha256": old_entry.get("jar_sha256"),
                "launch_configuration_id": launch_id,
                "runtime_status": "HOST_PANIC",
                "last_error": "Process crashed before producing JSON",
                "framebuffer_publications": 0,
                "executed_instructions": 0,
                "timing": {"total_wall_ms": 0.0},
                "fatal_guest_exception": False,
                "guest_exceptions_thrown": 0,
                "guest_exceptions_caught": 0,
                "guest_exceptions_uncaught": 0,
            }
        else:
            new_status = run_data.get("runtime_status", "UNKNOWN")

        run_data["file_id"] = old_entry.get("file_id")
        run_data["launch_configuration_id"] = launch_id
        new_results.append(run_data)

        trans_key = f"{old_status} -> {new_status}"
        transitions[trans_key] = transitions.get(trans_key, 0) + 1

        trans_record = {
            "launch_configuration_id": launch_id,
            "jar_path": jar_path,
            "old_status": old_status,
            "new_status": new_status,
            "old_wall_ms": old_entry.get("timing", {}).get("total_wall_ms", 0.0),
            "new_wall_ms": run_data.get("timing", {}).get("total_wall_ms", 0.0),
            "old_instructions": old_entry.get("executed_instructions", 0),
            "new_instructions": run_data.get("executed_instructions", 0),
            "guest_exceptions_thrown": run_data.get("guest_exceptions_thrown", 0),
            "guest_exceptions_caught": run_data.get("guest_exceptions_caught", 0),
            "guest_exceptions_uncaught": run_data.get("guest_exceptions_uncaught", 0),
            "fatal_guest_exception": run_data.get("fatal_guest_exception", False),
            "last_exception_class": run_data.get("last_exception_class"),
        }
        transitions_detail.append(trans_record)

        if (idx + 1) % 20 == 0 or idx + 1 == len(entries):
            elapsed = time.time() - t_start
            print(f"[{idx + 1}/100] {elapsed:.1f}s - {trans_key}")

    # Save outputs
    with open(OUT_JSON, "w") as f:
        json.dump({"results": new_results}, f, indent=2)

    diff_summary = {
        "total_configurations": len(entries),
        "transitions_summary": transitions,
        "transitions_detail": transitions_detail,
    }
    with open(DIFF_JSON, "w") as f:
        json.dump(diff_summary, f, indent=2)

    print("\nFixed 100 Run Complete!")
    print("Transitions Summary:")
    for k, v in sorted(transitions.items()):
        print(f"  {k}: {v}")

if __name__ == "__main__":
    main()
