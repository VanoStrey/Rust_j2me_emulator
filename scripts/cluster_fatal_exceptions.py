import json
from collections import defaultdict

with open("experiments/exp_phase45_exceptions_resources/reclassified_523.json") as f:
    data = json.load(f)

records = data.get("records", [])
fatal_runs = [r for r in records if r.get("fatal_guest_exception")]
print(f"Total evaluated records: {len(records)}")
print(f"Total fatal runs: {len(fatal_runs)}")

by_class = defaultdict(list)
for r in fatal_runs:
    cls = r.get("exception_class") or "Unknown"
    by_class[cls].append(r)

print("\n--- By Exception Class ---")
for cls, items in sorted(by_class.items(), key=lambda x: -len(x[1])):
    print(f"  {cls}: {len(items)} runs")

# Group by (class, location)
by_loc = defaultdict(list)
for r in fatal_runs:
    cls = r.get("exception_class") or "Unknown"
    loc = r.get("exception_location") or "Unknown"
    by_loc[(cls, loc)].append(r)

print(f"\nTotal distinct (class, location) pairs: {len(by_loc)}")
print("\n--- Top 20 distinct (class, location) clusters ---")
for (cls, loc), items in sorted(by_loc.items(), key=lambda x: -len(x[1]))[:20]:
    roms = [it["jar_path"].split("/")[-1] for it in items[:3]]
    print(f"  [{len(items)}] {cls} at {loc} -> {roms}")
