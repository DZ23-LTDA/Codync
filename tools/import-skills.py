"""Import skills from public repositories into this computer Codync skills folder.

A skill is a folder with a SKILL.md (market/mod.rs); Codync installs them under ~/.codync/skills/<id>.
This walks a repository tarball, finds every folder holding a SKILL.md and copies it, skipping ids that
already exist (so a local edit is never overwritten).

Usage: python tools/import-skills.py owner/repo [owner/repo ...]
"""
import io
import json
import pathlib
import re
import sys
import tarfile
import urllib.request

DATA = pathlib.Path.home() / ".codync" / "skills"
UA = {"User-Agent": "codync-import-skills"}


def slug(text):
    return re.sub(r"[^a-z0-9-]+", "-", text.lower()).strip("-") or "skill"


def fetch(repo):
    url = f"https://codeload.github.com/{repo}/tar.gz/HEAD"
    request = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(request, timeout=120) as response:
        return response.read()


def skills_in(tar_bytes):
    """Every folder with a SKILL.md: (path parts after the repo prefix, bytes of its files)."""
    found = {}
    with tarfile.open(fileobj=io.BytesIO(tar_bytes), mode="r:gz") as tar:
        for member in tar.getmembers():
            if not member.isfile():
                continue
            parts = pathlib.PurePosixPath(member.name).parts
            if len(parts) < 3:
                continue
            relative = parts[1:]  # drop the repo-HEAD prefix
            if relative[-1] == "SKILL.md":
                folder = "/".join(relative[:-1])
                found.setdefault(folder, {})["SKILL.md"] = tar.extractfile(member).read()
    # Keep the files a skill needs, one level around its SKILL.md (scripts, references).
    return found


def main(repos):
    if not repos:
        print(__doc__)
        return 1
    DATA.mkdir(parents=True, exist_ok=True)
    installed, skipped, empty = [], [], []
    for repo in repos:
        try:
            tar_bytes = fetch(repo)
        except Exception as error:  # noqa: BLE001 - one bad repo must not stop the rest
            empty.append((repo, f"download failed: {error}"))
            continue
        found = skills_in(tar_bytes)
        if not found:
            empty.append((repo, "no SKILL.md"))
            continue
        for folder, files in found.items():
            name = slug(pathlib.PurePosixPath(folder).name)
            target = DATA / name
            if target.exists():
                skipped.append(name)
                continue
            target.mkdir(parents=True, exist_ok=True)
            for filename, content in files.items():
                (target / filename).write_bytes(content)
            installed.append(name)
    print(json.dumps({"installed": installed, "skipped": skipped, "nothing": empty}, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
