import json
from collections import defaultdict

with open("experiments/exp_phase45_exceptions_resources/reclassified_523.json") as f:
    data = json.load(f)

records = data.get("records", [])
fatal_runs = [r for r in records if r.get("fatal_guest_exception")]

# 1. NoSuchElementException details
nsee = [r for r in fatal_runs if r.get("exception_class") == "java/util/NoSuchElementException"]
print(f"=== NoSuchElementException ({len(nsee)} runs) ===")
locs_nsee = defaultdict(list)
for r in nsee:
    locs_nsee[r.get("exception_location")].append(r["jar_path"].split("/")[-1])
for loc, jars in sorted(locs_nsee.items(), key=lambda x: -len(x[1])):
    print(f"  {loc} ({len(jars)} jars): {jars[:4]}")

# 2. ArithmeticException details
ae = [r for r in fatal_runs if r.get("exception_class") == "java/lang/ArithmeticException"]
print(f"\n=== ArithmeticException ({len(ae)} runs) ===")
locs_ae = defaultdict(list)
for r in ae:
    locs_ae[r.get("exception_location")].append(r["jar_path"].split("/")[-1])
for loc, jars in sorted(locs_ae.items(), key=lambda x: -len(x[1])):
    print(f"  {loc} ({len(jars)} jars): {jars[:4]}")

# 3. Top NPE locations
npe = [r for r in fatal_runs if r.get("exception_class") == "java/lang/NullPointerException"]
print(f"\n=== Top NPE Locations ({len(npe)} runs total) ===")
locs_npe = defaultdict(list)
for r in npe:
    locs_npe[r.get("exception_location")].append(r["jar_path"].split("/")[-1])
for loc, jars in sorted(locs_npe.items(), key=lambda x: -len(x[1]))[:15]:
    print(f"  {loc} ({len(jars)} jars): {jars[:4]}")
