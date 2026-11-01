#!/usr/bin/env python3
import os
import subprocess
import shutil

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
STUBS_DIR = os.path.join(BASE_DIR, "stubs")
STUBS_BIN = os.path.join(STUBS_DIR, "bin")
SRC_DIR = os.path.join(BASE_DIR, "active_render", "src")
BIN_DIR = os.path.join(BASE_DIR, "active_render", "bin")
JARS_DIR = os.path.join(BASE_DIR, "active_render", "jars")

os.makedirs(BIN_DIR, exist_ok=True)
os.makedirs(JARS_DIR, exist_ok=True)

# 1. Compile all fixtures against stubs
java_files = [os.path.join(SRC_DIR, f) for f in os.listdir(SRC_DIR) if f.endswith(".java")]
print(f"Compiling {len(java_files)} fixture java files...")

cmd = [
    "javac",
    "-source", "8",
    "-target", "8",
    "-cp", STUBS_BIN,
    "-d", BIN_DIR,
] + java_files

subprocess.run(cmd, check=True)

fixtures = [
    "Fixture1DrawNoFlush",
    "Fixture2FullFlush",
    "Fixture3RegionalFlush",
    "Fixture4RepeatedGetGraphics",
    "Fixture5TwoGameCanvases",
    "Fixture6SwitchingDisplayable",
    "Fixture7FullscreenAndOrdinaryCanvas",
    "Fixture8PaintAndActiveRender",
    "Fixture9SuppressKeyEvents",
    "Fixture10DirectGraphics",
]

for fix in fixtures:
    temp_pkg = os.path.join(BIN_DIR, f"temp_{fix}")
    if os.path.exists(temp_pkg):
        shutil.rmtree(temp_pkg)
    os.makedirs(os.path.join(temp_pkg, "META-INF"), exist_ok=True)

    manifest_content = f"""Manifest-Version: 1.0
MicroEdition-Configuration: CLDC-1.1
MicroEdition-Profile: MIDP-2.0
MIDlet-Name: {fix}
MIDlet-Version: 1.0.0
MIDlet-Vendor: FixtureVendor
MIDlet-1: {fix},,fixtures.{fix}
"""
    manifest_path = os.path.join(temp_pkg, "META-INF", "MANIFEST.MF")
    with open(manifest_path, "w") as f:
        f.write(manifest_content)

    # Only copy fixture classes (stubs are provided by the emulator runtime)
    fix_class_dir = os.path.join(temp_pkg, "fixtures")
    os.makedirs(fix_class_dir, exist_ok=True)
    for f in os.listdir(os.path.join(BIN_DIR, "fixtures")):
        if f.startswith(fix):
            shutil.copy2(os.path.join(BIN_DIR, "fixtures", f), os.path.join(fix_class_dir, f))

    jar_path = os.path.join(JARS_DIR, f"{fix}.jar")
    # Package jar
    subprocess.run(["jar", "cfm", jar_path, manifest_path, "-C", temp_pkg, "."], check=True)
    print(f"Created {jar_path}")
    shutil.rmtree(temp_pkg)

print("All fixtures built successfully!")
