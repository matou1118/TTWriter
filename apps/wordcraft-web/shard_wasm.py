#!/usr/bin/env python3
"""Shard a trunk-built _bg.wasm into <=18MB pieces + manifest.json.

Used by pages.yml CI (local fallback in _site/app/shards/) and by
shard-wasm.ps1 (local builds). jsdelivr gh mirror caps single files at
~20MB; our wasm is ~55MB so we split. The index.html fetch-intercept
reassembles before wasm-bindgen inits.

Usage: python3 shard_wasm.py <dir-containing-_bg.wasm> [--out <shards-dir>]
"""
import os, sys, json, math, glob

def main():
    if len(sys.argv) < 2:
        sys.exit("usage: shard_wasm.py <wasm-dir> [--out <shards-dir>]")
    wasm_dir = sys.argv[1]
    out = None
    if "--out" in sys.argv:
        out = sys.argv[sys.argv.index("--out") + 1]

    candidates = sorted(glob.glob(os.path.join(wasm_dir, "*_bg.wasm")))
    if not candidates:
        sys.exit(f"no *_bg.wasm in {wasm_dir}")
    wasm_path = candidates[0]
    with open(wasm_path, "rb") as f:
        data = f.read()

    shard = 18 * 1048576  # 18MB, safely under jsdelivr's ~20MB cap
    n = max(1, math.ceil(len(data) / shard))
    shards_dir = out or os.path.join(wasm_dir, "shards")
    os.makedirs(shards_dir, exist_ok=True)

    files = [f"{i}.bin" for i in range(n)]
    for i in range(n):
        with open(os.path.join(shards_dir, f"{i}.bin"), "wb") as f:
            f.write(data[i * shard:(i + 1) * shard])

    manifest = {"size": len(data), "shards": files, "source": os.path.basename(wasm_path)}
    with open(os.path.join(shards_dir, "manifest.json"), "w") as f:
        json.dump(manifest, f)

    print(f"sharded {len(data)} bytes -> {n} shards in {shards_dir}")

if __name__ == "__main__":
    main()
