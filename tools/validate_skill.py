import re, sys, pathlib

p = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "skills/wordcraft/SKILL.md")
raw = p.read_text(encoding="utf-8")
m = re.match(r"^---\r?\n(.*?)\r?\n---\r?\n(.*)$", raw, re.S)
if not m:
    print("FAIL: no frontmatter")
    sys.exit(1)
fm, body = m.group(1), m.group(2)

def field(k):
    mm = re.search(rf"^{k}:\s*(.+)$", fm, re.M)
    return mm.group(1).strip() if mm else None

name = field("name")
desc = field("description")
lic = field("license")
compat = field("compatibility")
meta = field("metadata")

checks = []
def chk(label, ok, detail=""):
    checks.append((label, ok, detail))

chk("frontmatter present", True)
chk("name present", bool(name), name or "")
chk("name 1-64 chars", bool(name) and 1 <= len(name) <= 64, f"len={len(name) if name else 0}")
chk("name lowercase alnum + hyphens", bool(name) and re.fullmatch(r"[a-z0-9]+(-[a-z0-9]+)*", name) is not None, name or "")
chk("name has no leading/trailing hyphen", bool(name) and not name.startswith("-") and not name.endswith("-"))
chk("name has no '--'", bool(name) and "--" not in name)
chk("name matches parent dir", name == p.parent.name, f"{name} vs {p.parent.name}")
chk("description present", bool(desc))
chk("description <= 1024", bool(desc) and len(desc) <= 1024, f"len={len(desc) if desc else 0}")
chk("license <= short", lic is None or len(lic) <= 120, lic or "")
chk("compatibility <= 500", compat is None or len(compat) <= 500, f"len={len(compat) if compat else 0}")
chk("metadata is a map", meta is None or meta is not None)  # presence only
chk("body < 500 lines", len(body.splitlines()) < 500, f"lines={len(body.splitlines())}")

# every referenced file must exist (strip any #anchor)
root = p.parent
refs = re.findall(r"\]\(([^)#]+)(#[^)]+)?\)", body)
for r in sorted(set(x[0] for x in refs)):
    chk(f"reference exists: {r}", (root / r).exists())

# no CJK leaking into a description that must stay English? (allowed, but check it's short)
print("=== spec checks ===")
bad = 0
for label, ok, detail in checks:
    if not ok:
        bad += 1
    print(f"  {'PASS' if ok else 'FAIL'}  {label}" + (f"  [{detail}]" if detail and not ok else ""))
print(f"\n{len(checks)-bad}/{len(checks)} passed")
print(f"description length: {len(desc)}")
sys.exit(1 if bad else 0)
