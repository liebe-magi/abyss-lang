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

def get_workspace_dep_versions(cargo_path):
    """Return {name: version} for intra-workspace crates listed in
    [workspace.dependencies] (those pointing at a local `crates/…` path).
    These versions must match [workspace.package].version; otherwise a
    release-time `cargo publish` will produce stale dependency requirements
    and may fail or publish incorrect metadata.
    """
    try:
        with open(cargo_path, 'r') as f:
            content = f.read()
        section_match = re.search(
            r'^\[workspace\.dependencies\]\s*\n(.*?)(?=^\[|\Z)',
            content,
            re.MULTILINE | re.DOTALL,
        )
        if not section_match:
            return {}
        section = section_match.group(1)
        versions = {}
        entry_re = re.compile(
            r'^\s*([\w-]+)\s*=\s*\{([^}]+)\}\s*$',
            re.MULTILINE,
        )
        for m in entry_re.finditer(section):
            name = m.group(1)
            body = m.group(2)
            path_match = re.search(r'path\s*=\s*"(crates/[^"]+)"', body)
            version_match = re.search(r'version\s*=\s*"([^"]+)"', body)
            if path_match and version_match:
                versions[name] = version_match.group(1)
        return versions
    except Exception as e:
        print(f"Error reading Cargo.toml for workspace dependencies: {e}")
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
    dep_versions = get_workspace_dep_versions(cargo_path)

    print(f"Cargo version: {cargo_version}")
    print(f"Package version: {package_version}")
    if dep_versions:
        print(f"Workspace deps: {dep_versions}")

    if cargo_version != package_version:
        print(f"Error: Versions do not match! Cargo.toml has '{cargo_version}' but package.json has '{package_version}'")
        print("Please update the version in one of these files to match the other.")
        sys.exit(1)

    mismatched = [
        (name, ver) for name, ver in dep_versions.items() if ver != cargo_version
    ]
    if mismatched:
        print(
            f"Error: [workspace.dependencies] entries disagree with [workspace.package].version ('{cargo_version}'):"
        )
        for name, ver in mismatched:
            print(f"  - {name}: '{ver}'")
        print(
            "Bump [workspace.dependencies].*.version in lockstep with [workspace.package].version, "
            "otherwise `cargo publish` will emit stale dependency requirements."
        )
        sys.exit(1)

    print("Versions match.")
    sys.exit(0)

if __name__ == "__main__":
    main()
