#!/usr/bin/env python3
from sys import argv, stderr

PROFILES: dict[str, list[str]] = {
    "base": ["gpio"],
    "roland": ["gpio", "roland", "camloc"],
}

os = argv[1].split("-")[0]
name = argv[2]
features = PROFILES[name]

if os == "ubuntu":
    features.append("backend")

print("f=" + ",".join(features), file=stderr)
print("f=" + ",".join(features))
