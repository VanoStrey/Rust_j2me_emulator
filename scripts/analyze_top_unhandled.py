import json
from collections import Counter

results_file = "experiments/exp_phase46_stability_blockers/rerun_49/post_fix_226_results.json"
with open(results_file) as f:
    data = json.load(f)

unhandled_counter = Counter()
recent_counter = Counter()
last_call_counter = Counter()
exceptions_counter = Counter()

total_exceptions = 0
for entry in data:
    status = entry.get("new_status")
    if status == "GUEST_EXCEPTION":
        total_exceptions += 1
        exc = entry.get("last_exception_class", "unknown")
        exceptions_counter[exc] += 1

        unhandled = entry.get("unhandled_natives", [])
        for u in unhandled:
            unhandled_counter[u] += 1

        recent = entry.get("recent_native_calls", [])
        for r in recent:
            sig = r.split(" -> ")[0] if " -> " in r else r
            recent_counter[sig] += 1
        if recent:
            last = recent[-1]
            last_sig = last.split(" -> ")[0] if " -> " in last else last
            last_call_counter[last_sig] += 1

print(f"Total remaining GUEST_EXCEPTION: {total_exceptions}")
print("\nExceptions count:")
for exc, cnt in exceptions_counter.most_common(10):
    print(f"  {exc}: {cnt}")

print("\n--- TOP-15 UNHANDLED NATIVES (Present in failing configs) ---")
for sig, cnt in unhandled_counter.most_common(15):
    print(f"  {cnt:3d}  {sig}")

print("\n--- TOP-15 RECENT NATIVE CALLS (Called before crash) ---")
for sig, cnt in recent_counter.most_common(15):
    print(f"  {cnt:3d}  {sig}")

print("\n--- TOP-15 EXACT LAST NATIVE CALL (Immediately preceding crash) ---")
for sig, cnt in last_call_counter.most_common(15):
    print(f"  {cnt:3d}  {sig}")
