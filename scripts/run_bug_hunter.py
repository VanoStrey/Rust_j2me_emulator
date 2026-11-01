#!/usr/bin/env python3
"""
Python CLI runner for Phase 5.12 Bug-Hunting & Behavioral Fuzzing Pipeline.
Invokes scripts/run_bug_hunter.mjs and streams output in real time.
"""

import os
import sys
import subprocess

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    mjs_path = os.path.join(root_dir, "scripts", "run_bug_hunter.mjs")

    print("[BugHunter Runner] Launching Smart Fuzzing Pipeline via Node.js...")
    result = subprocess.run(["node", mjs_path], cwd=root_dir)
    sys.exit(result.returncode)

if __name__ == "__main__":
    main()
