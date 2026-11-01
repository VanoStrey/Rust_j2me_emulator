import json
from collections import Counter

with open("experiments/phase55_148_signatures.json", "r") as f:
    data = json.load(f)

print(f"Total entries: {len(data)}")

# Count by exception class
exc_counter = Counter(d.get("exception_class", "Unknown") for d in data)
print("\n--- Exception Classes ---")
for exc, count in exc_counter.most_common():
    print(f"{exc:45} : {count}")

# Extract top detailed patterns from message
# Messages format:
# Null pointer dereference during GetField { class_name: "ba", field_name: "a", descriptor: "Lcb;" } at n.a(...)
# Array index out of bounds during AALoad at b.a(II)V (pc=6)
# / by zero during IDiv at ...

patterns = Counter()
for d in data:
    msg = d.get("message", "")
    cls = d.get("exception_class", "").split("/")[-1]
    loc = d.get("location", "")
    
    # Simplify pattern
    if "during GetField" in msg:
        # extract field
        parts = msg.split("GetField {")
        if len(parts) > 1:
            field_part = parts[1].split("}")[0].strip()
            patterns[f"NPE during GetField {{{field_part}}}"] += 1
        else:
            patterns["NPE during GetField"] += 1
    elif "during InvokeVirtual" in msg:
        parts = msg.split("InvokeVirtual {")
        if len(parts) > 1:
            method_part = parts[1].split("}")[0].strip()
            patterns[f"NPE during InvokeVirtual {{{method_part}}}"] += 1
        else:
            patterns["NPE during InvokeVirtual"] += 1
    elif "during InvokeInterface" in msg:
        parts = msg.split("InvokeInterface {")
        if len(parts) > 1:
            method_part = parts[1].split("}")[0].strip()
            patterns[f"NPE during InvokeInterface {{{method_part}}}"] += 1
        else:
            patterns["NPE during InvokeInterface"] += 1
    elif "during AALoad" in msg:
        patterns["AIOOBE during AALoad"] += 1
    elif "during BALoad" in msg:
        patterns["AIOOBE during BALoad"] += 1
    elif "during IALoad" in msg:
        patterns["AIOOBE during IALoad"] += 1
    elif "during CALoad" in msg:
        patterns["AIOOBE during CALoad"] += 1
    elif "during IDiv" in msg:
        patterns["ArithmeticException: / by zero (IDiv)"] += 1
    elif "during IRem" in msg:
        patterns["ArithmeticException: / by zero (IRem)"] += 1
    elif "during LDiv" in msg:
        patterns["ArithmeticException: / by zero (LDiv)"] += 1
    elif "during AALoad/AAStore" in msg:
        patterns["AIOOBE during Array Load/Store"] += 1
    else:
        patterns[f"{cls}: {msg[:60]}"] += 1

print("\n--- TOP Detailed Crash Signatures ---")
for pat, count in patterns.most_common(25):
    print(f"{count:3}x | {pat}")
