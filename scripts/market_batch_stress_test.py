#!/usr/bin/env python3
"""
scripts/market_batch_stress_test.py
Phase 5.6 Automated Market Batch Stress Test & Compatibility Suite

Recursively scans J2ME market collections (e.g. 7,288 JARs), executes headless
CLI verification with timeouts, aggregates failure signatures, and tracks
incremental progress with resume support.
"""

import os
import sys
import json
import time
import argparse
import subprocess
from pathlib import Path
from concurrent.futures import ProcessPoolExecutor, as_completed
from collections import Counter, defaultdict

def scan_jars(market_dir):
    """Recursively discover all .jar files in market_dir."""
    jar_files = []
    # If roms directory exists, prioritize roms/Games, then other roms, then root
    p = Path(market_dir)
    for root, _, files in os.walk(p):
        # Exclude tools/ directory containing utility jars like cfr.jar, freej2me.jar
        if "/tools" in root or root.endswith("/tools"):
            continue
        for f in files:
            if f.lower().endswith(".jar"):
                jar_files.append(os.path.abspath(os.path.join(root, f)))
    jar_files.sort()
    return jar_files

def run_single_jar(item):
    """Worker task executing a single game via CLI."""
    idx, jar_path, exe_path, ticks, timeout_sec, tmp_dir = item
    tmp_json = os.path.join(tmp_dir, f"market_run_{os.getpid()}_{idx}_{int(time.time()*1000)%100000}.json")

    cmd = [
        exe_path,
        "--run-single", jar_path,
        "--ticks", str(ticks),
        "--output-json", tmp_json
    ]

    t0 = time.time()
    try:
        proc = subprocess.run(
            cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=timeout_sec
        )
        elapsed = time.time() - t0

        if os.path.exists(tmp_json):
            try:
                with open(tmp_json, "r", encoding="utf-8") as f:
                    data = json.load(f)
                try:
                    os.remove(tmp_json)
                except OSError:
                    pass

                runtime_status = data.get("runtime_status", "UNKNOWN")
                exit_reason = data.get("exit_reason", "")
                last_err = data.get("last_error") or ""

                # Classify into canonical categories
                if runtime_status == "COMPLETED_WINDOW":
                    category = "COMPLETED_WINDOW"
                elif runtime_status == "GUEST_EXCEPTION":
                    category = "GUEST_EXCEPTION"
                elif runtime_status == "LOAD_ERROR" or "I/O Error" in last_err or "ZIP" in last_err or "Manifest" in last_err:
                    category = "PARSER_ERROR"
                else:
                    category = runtime_status

                return {
                    "index": idx,
                    "jar_path": jar_path,
                    "jar_name": os.path.basename(jar_path),
                    "category": category,
                    "runtime_status": runtime_status,
                    "observed_ui": data.get("observed_ui", "UNKNOWN"),
                    "paint_calls": data.get("paint_calls", 0),
                    "instructions": data.get("executed_instructions", 0),
                    "exception_class": data.get("last_exception_class"),
                    "exception_message": data.get("last_exception_message"),
                    "exception_location": data.get("last_exception_location"),
                    "thread_stacks": data.get("active_thread_stacks", []),
                    "exit_reason": exit_reason,
                    "last_error": last_err,
                    "elapsed_sec": round(elapsed, 3),
                    "success": category in ("COMPLETED_WINDOW", "WALL_TIMEOUT")
                }
            except Exception as parse_e:
                return {
                    "index": idx,
                    "jar_path": jar_path,
                    "jar_name": os.path.basename(jar_path),
                    "category": "PARSER_ERROR",
                    "runtime_status": "CORRUPT_OUTPUT",
                    "last_error": f"Failed to parse emulator output: {parse_e}",
                    "elapsed_sec": round(elapsed, 3),
                    "success": False
                }
        else:
            return {
                "index": idx,
                "jar_path": jar_path,
                "jar_name": os.path.basename(jar_path),
                "category": "PARSER_ERROR",
                "runtime_status": "LOAD_ERROR",
                "last_error": f"Emulator exited with code {proc.returncode} without output JSON",
                "elapsed_sec": round(elapsed, 3),
                "success": False
            }
    except subprocess.TimeoutExpired:
        elapsed = time.time() - t0
        return {
            "index": idx,
            "jar_path": jar_path,
            "jar_name": os.path.basename(jar_path),
            "category": "TIMEOUT_DEADLOCK",
            "runtime_status": "WALL_TIMEOUT",
            "last_error": f"Subprocess timed out after {timeout_sec}s",
            "elapsed_sec": round(elapsed, 3),
            "success": False
        }
    except Exception as host_e:
        elapsed = time.time() - t0
        return {
            "index": idx,
            "jar_path": jar_path,
            "jar_name": os.path.basename(jar_path),
            "category": "HOST_PANIC",
            "runtime_status": "HOST_PANIC",
            "last_error": str(host_e),
            "elapsed_sec": round(elapsed, 3),
            "success": False
        }

def save_report(results, total_scanned, out_json_path, out_md_path, start_time):
    """Aggregate statistics and write JSON and Markdown reports."""
    categories = Counter()
    status_counts = Counter()
    exception_classes = Counter()
    detail_signatures = Counter()
    parser_errors = Counter()
    timeout_locations = Counter()
    total_instructions = 0
    total_paints = 0

    for r in results:
        cat = r.get("category", "UNKNOWN")
        categories[cat] += 1
        st = r.get("runtime_status", "UNKNOWN")
        status_counts[st] += 1
        total_instructions += r.get("instructions", 0)
        total_paints += r.get("paint_calls", 0)

        if cat == "GUEST_EXCEPTION":
            exc = r.get("exception_class") or "UnknownException"
            exception_classes[exc] += 1
            msg = r.get("exception_message") or ""
            loc = r.get("exception_location") or ""
            sig = f"{exc}: {msg} at {loc}".strip()
            detail_signatures[sig] += 1
        elif cat == "PARSER_ERROR":
            err = r.get("last_error") or "Unknown parser error"
            short_err = err.split("\n")[0][:80]
            parser_errors[short_err] += 1
        elif cat in ("TIMEOUT_DEADLOCK", "WALL_TIMEOUT"):
            stacks = r.get("thread_stacks") or []
            if stacks:
                for s in stacks:
                    parts = s.split("]: ")
                    loc = parts[1] if len(parts) > 1 else s
                    timeout_locations[loc] += 1
            else:
                last_err = r.get("last_error") or "Subprocess timeout"
                timeout_locations[last_err] += 1

    total_tested = len(results)
    elapsed_total = time.time() - start_time

    summary = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "total_collection_jars": total_scanned,
        "total_tested": total_tested,
        "elapsed_seconds": round(elapsed_total, 2),
        "categories": dict(categories),
        "runtime_status": dict(status_counts),
        "total_instructions": total_instructions,
        "total_paint_calls": total_paints,
        "top_exception_classes": dict(exception_classes.most_common(20)),
        "top_detail_signatures": dict(detail_signatures.most_common(25)),
        "top_parser_errors": dict(parser_errors.most_common(10)),
        "top_timeout_locations": dict(timeout_locations.most_common(20)),
        "results": results
    }

    os.makedirs(os.path.dirname(os.path.abspath(out_json_path)), exist_ok=True)
    with open(out_json_path, "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2)

    # Generate Markdown
    md_lines = [
        "# J2ME Market Batch Compatibility & Stress Test Audit Report",
        f"**Date:** {summary['timestamp']} | **Total Games Tested:** {total_tested}/{total_scanned} | **Elapsed:** {summary['elapsed_seconds']}s\n",
        "## 1. High-Level Category Distribution",
        "| Category | Count | Percentage |",
        "|:---|:---:|:---:|"
    ]
    for cat, count in categories.most_common():
        pct = (count / total_tested * 100) if total_tested > 0 else 0
        md_lines.append(f"| **{cat}** | {count} | {pct:.1f}% |")

    md_lines.extend([
        "\n## 2. Top Uncaught Guest Exceptions",
        "| Exception Class | Count | % of Exceptions |",
        "|:---|:---:|:---:|"
    ])
    total_exceptions = categories.get("GUEST_EXCEPTION", 0)
    for exc, count in exception_classes.most_common(15):
        pct = (count / total_exceptions * 100) if total_exceptions > 0 else 0
        md_lines.append(f"| `{exc}` | {count} | {pct:.1f}% |")

    md_lines.extend([
        "\n## 3. Top Detailed Exception Signatures",
        "| Occurrences | Signature & Location |",
        "|:---:|:---|"
    ])
    for sig, count in detail_signatures.most_common(20):
        md_lines.append(f"| **{count}x** | `{sig}` |")

    md_lines.extend([
        "\n## 4. Top Parser & Loader Errors",
        "| Occurrences | Error Description |",
        "|:---:|:---|"
    ])
    for err, count in parser_errors.most_common(10):
        md_lines.append(f"| **{count}x** | `{err}` |")

    md_lines.extend([
        "\n## 5. Top 20 Timeout & Deadlock Locations (Hang Slices)",
        "| Occurrences | Class / Method / Stack Location |",
        "|:---:|:---|"
    ])
    for loc, count in timeout_locations.most_common(20):
        md_lines.append(f"| **{count}x** | `{loc}` |")

    if out_md_path:
        os.makedirs(os.path.dirname(os.path.abspath(out_md_path)), exist_ok=True)
        with open(out_md_path, "w", encoding="utf-8") as f:
            f.write("\n".join(md_lines) + "\n")

    return summary

def main():
    parser = argparse.ArgumentParser(description="J2ME Market Batch Compatibility Runner")
    parser.add_argument("--market-dir", default="/Users/foolvan/Desktop/j2me_market", help="Directory containing .jar files")
    parser.add_argument("--limit", type=int, default=None, help="Limit number of games to test")
    parser.add_argument("--ticks", type=int, default=300, help="Frames/ticks to execute per game")
    parser.add_argument("--timeout", type=float, default=16.0, help="Subprocess timeout in seconds")
    parser.add_argument("--workers", type=int, default=None, help="Concurrent worker count")
    parser.add_argument("--resume", action="store_true", help="Resume from previous audit report")
    parser.add_argument("--output-json", default="experiments/market_audit_report.json", help="Path to output JSON report")
    parser.add_argument("--output-md", default="experiments/market_audit_report.md", help="Path to output Markdown report")
    args = parser.parse_args()

    exe_path = os.path.abspath("./target/release/j2me-cli")
    if not os.path.exists(exe_path):
        print(f"[ERROR] Executable not found at {exe_path}. Build it first with cargo build --release -p j2me-cli")
        sys.exit(1)

    print(f"[Scanner] Scanning for JAR files in: {args.market_dir}...")
    jars = scan_jars(args.market_dir)
    print(f"[Scanner] Discovered {len(jars)} JAR files in market collection.")

    if not jars:
        print("[ERROR] No JAR files found.")
        sys.exit(1)

    tested_results = []
    already_tested_paths = set()

    if args.resume and os.path.exists(args.output_json):
        try:
            with open(args.output_json, "r", encoding="utf-8") as f:
                prev_data = json.load(f)
            prev_results = prev_data.get("results", [])
            for r in prev_results:
                tested_results.append(r)
                already_tested_paths.add(r["jar_path"])
            print(f"[Resume] Loaded {len(tested_results)} previously tested games from {args.output_json}")
        except Exception as e:
            print(f"[Warning] Failed to read resume file: {e}")

    # Filter out already tested
    pending_jars = [j for j in jars if j not in already_tested_paths]
    if args.limit:
        target_count = min(args.limit, len(jars))
        remaining_slots = max(0, target_count - len(tested_results))
        pending_jars = pending_jars[:remaining_slots]

    print(f"[Queue] Pending games to test: {len(pending_jars)} (Total tested so far: {len(tested_results)})")

    if not pending_jars:
        print("[Queue] No pending games to run. Generating report from existing results...")
        save_report(tested_results, len(jars), args.output_json, args.output_md, time.time())
        print(f"[Report] Saved JSON: {args.output_json}")
        print(f"[Report] Saved MD: {args.output_md}")
        return

    tmp_dir = os.path.abspath("experiments/market_tmp")
    os.makedirs(tmp_dir, exist_ok=True)

    workers = args.workers or min(8, os.cpu_count() or 4)
    print(f"[Runner] Launching batch runner with {workers} parallel workers (timeout={args.timeout}s, ticks={args.ticks})...")

    items = [
        (len(tested_results) + i, jar_path, exe_path, args.ticks, args.timeout, tmp_dir)
        for i, jar_path in enumerate(pending_jars)
    ]

    start_time = time.time()
    last_flush = time.time()

    with ProcessPoolExecutor(max_workers=workers) as executor:
        futures = {executor.submit(run_single_jar, it): it for it in items}
        completed_count = 0

        for fut in as_completed(futures):
            res = fut.result()
            tested_results.append(res)
            completed_count += 1

            if completed_count % 25 == 0 or completed_count == len(items):
                cat = res.get("category", "UNKNOWN")
                print(f"[{completed_count}/{len(items)}] {res['jar_name'][:30]:30} | {cat:16} | instructions={res.get('instructions',0)} | time={res.get('elapsed_sec',0)}s")

            # Periodic checkpoint flush every 50 games or 30 seconds
            if completed_count % 50 == 0 or time.time() - last_flush > 30:
                save_report(tested_results, len(jars), args.output_json, args.output_md, start_time)
                last_flush = time.time()

    # Final report
    summary = save_report(tested_results, len(jars), args.output_json, args.output_md, start_time)

    print("\n" + "=" * 80)
    print("                     MARKET BATCH AUDIT SUMMARY                        ")
    print("=" * 80)
    print(f"Total Games Tested: {len(tested_results)} / {len(jars)}")
    print(f"Elapsed Time:       {summary['elapsed_seconds']}s")
    print("-" * 80)
    for cat, count in summary["categories"].items():
        pct = count / len(tested_results) * 100
        print(f"  {cat:22}: {count:5} ({pct:.1f}%)")
    print("=" * 80)
    print(f"[Report] Saved JSON: {args.output_json}")
    print(f"[Report] Saved MD:   {args.output_md}")

if __name__ == "__main__":
    main()
