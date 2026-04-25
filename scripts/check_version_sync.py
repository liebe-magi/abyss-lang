#!/usr/bin/env python3
import sys
import json
import os
import re

def get_cargo_version(cargo_path):
    try:
        with open(cargo_path, 'r') as f:
            content = f.read()
            # The canonical version lives under [workspace.package] in the root
            # Cargo.toml; individual crates inherit it via `version.workspace = true`.
            # Scope the search to the [workspace.package] section so unrelated
            # version specifiers (e.g. entries in [workspace.dependencies]) do not
            # shadow it.
            section_match = re.search(
                r'^\[workspace\.package\]\s*\n(.*?)(?=^\[|\Z)',
                content,
                re.MULTILINE | re.DOTALL,
            )
            if not section_match:
                print("Could not find [workspace.package] section in Cargo.toml")
                sys.exit(1)
            match = re.search(
                r'^version\s*=\s*"(\d+\.\d+\.\d+(?:-[\w.]+)?)"',
                section_match.group(1),
                re.MULTILINE,
            )
            if match:
                return match.group(1)
            else:
                print("Could not find version in [workspace.package]")
                sys.exit(1)
    except Exception as e:
        print(f"Error reading Cargo.toml: {e}")
        sys.exit(1)

def get_package_json_version(package_path):
    try:
        with open(package_path, 'r') as f:
            data = json.load(f)
            if 'version' not in data:
                print("Could not find version in package.json")
                sys.exit(1)
            return data['version']
    except Exception as e:
        print(f"Error reading package.json: {e}")
        sys.exit(1)

def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
    cargo_path = os.path.join(root_dir, 'Cargo.toml')
    package_path = os.path.join(root_dir, 'editors/code/package.json')

    if not os.path.exists(cargo_path):
        print(f"Cargo.toml not found at {cargo_path}")
        sys.exit(1)
    
    if not os.path.exists(package_path):
        print(f"package.json not found at {package_path}")
        sys.exit(1)

    cargo_version = get_cargo_version(cargo_path)
    package_version = get_package_json_version(package_path)

    print(f"Cargo version: {cargo_version}")
    print(f"Package version: {package_version}")

    if cargo_version != package_version:
        print(f"Error: Versions do not match! Cargo.toml has '{cargo_version}' but package.json has '{package_version}'")
        print("Please update the version in one of these files to match the other.")
        sys.exit(1)
    
    print("Versions match.")
    sys.exit(0)

if __name__ == "__main__":
    main()
