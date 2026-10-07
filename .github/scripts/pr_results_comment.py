#!/usr/bin/env python3
"""Post test and benchmark results on a pull request.

Looks for an existing comment that starts with a marker and updates it.
A later commit on the same pull request replaces that comment's body.
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

MARKER = "<!-- json-traits-ci-results -->"
ANSI = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")
MAX_LOG = 20000
BOT_LOGIN = "github-actions[bot]"


def clean(text: str) -> str:
    return ANSI.sub("", text).replace("\r\n", "\n").strip()


def read_log(path: Path) -> str | None:
    if not path.is_file():
        return None
    return clean(path.read_text(errors="replace"))


def fence(text: str) -> str:
    ticks = "```"
    while ticks in text:
        ticks += "`"
    return f"{ticks}text\n{text.rstrip()}\n{ticks}"


def clip(text: str) -> str:
    if len(text) <= MAX_LOG:
        return text
    return text[:MAX_LOG].rstrip() + "\n\n… output truncated …"


def test_highlights(log: str) -> str:
    lines = log.splitlines()
    kept: list[str] = []
    failing = False
    for line in lines:
        if line.startswith("failures:"):
            failing = True
        if line.startswith("test result:") or " FAILED" in line:
            kept.append(line)
        elif failing and line.strip():
            kept.append(line)
            if line.startswith("test result:"):
                failing = False
    return "\n".join(kept)


def benchmark_highlights(log: str, compare: bool = False) -> str:
    lines = log.splitlines()
    kept: list[str] = []
    pending_name: str | None = None
    for line in lines:
        stripped = line.strip()
        if not stripped or stripped.startswith("Benchmarking ") or stripped.startswith("Gnuplot "):
            continue
        if stripped.startswith("Warning:"):
            continue
        compared = compare and (
            stripped.startswith("change:")
            or "Performance has " in stripped
            or (("%" in stripped) and ("time:" in stripped or stripped.startswith("thrpt:")))
        )
        if compared:
            kept.append(stripped)
            continue
        if stripped.startswith("change:"):
            pending_name = None
            continue
        if ("time:" in stripped or stripped.startswith("thrpt:")) and "%" not in stripped:
            if pending_name:
                kept.append(pending_name)
                pending_name = None
            kept.append(stripped)
            continue
        if stripped.startswith("Found ") and "outlier" in stripped:
            kept.append(stripped)
            continue
        if line[:1].strip() and not line.startswith(" "):
            pending_name = stripped
    return "\n".join(kept)


def section(title: str, conclusion: str, log: str | None, highlights, details: str) -> str:
    status = conclusion if conclusion else "unknown"
    parts = [f"### {title} — {status}", ""]
    if log:
        summary = highlights(log)
        if summary:
            parts.append(fence(summary))
            parts.append("")
        parts.append("<details>")
        parts.append(f"<summary>{details}</summary>")
        parts.append("")
        parts.append(fence(clip(log)))
        parts.append("")
        parts.append("</details>")
    else:
        parts.append("No output was saved.")
    return "\n".join(parts)


def linked_sha(sha: str, url: str) -> str:
    short = sha[:7] if sha else "unknown"
    return f"[`{short}`]({url})" if url and sha else f"`{short}`"


def read_bench_status(path: Path) -> dict[str, str]:
    if not path.is_file():
        return {}
    status: dict[str, str] = {}
    for line in path.read_text(errors="replace").splitlines():
        if "=" not in line:
            continue
        name, code = line.split("=", 1)
        status[name.strip()] = "success" if code.strip() == "0" else "failure"
    return status


def build_comment(
    test_log: str | None,
    main_log: str | None,
    pr_log: str | None,
    bench_status: dict[str, str],
) -> str:
    head = linked_sha(os.environ.get("HEAD_SHA", ""), os.environ.get("COMMIT_URL", ""))
    base = linked_sha(os.environ.get("BASE_SHA", ""), os.environ.get("BASE_URL", ""))
    run_url = os.environ.get("RUN_URL", "")
    run = f"[workflow run]({run_url})" if run_url else "workflow run"
    fallback = os.environ.get("BENCH_CONCLUSION", "")
    parts = [
        MARKER,
        "## Tests and benchmarks",
        f"Pull request {head} · main {base} · {run}",
        "Both benchmark runs use the same machine. The pull request run includes Criterion's comparison against main.",
        section(
            "Tests",
            os.environ.get("TEST_CONCLUSION", ""),
            test_log,
            test_highlights,
            "Full test output",
        ),
        section(
            f"Benchmarks on main ({base})",
            bench_status.get("main", fallback),
            main_log,
            benchmark_highlights,
            "Full main benchmark output",
        ),
        section(
            f"Benchmarks on this pull request ({head})",
            bench_status.get("pr", fallback),
            pr_log,
            lambda log: benchmark_highlights(log, compare=True),
            "Full pull request benchmark output",
        ),
    ]
    body = "\n\n".join(parts)
    # GitHub rejects issue comments larger than 65536 characters.
    if len(body) > 65000:
        body = body[:65000].rstrip() + "\n\n… comment truncated …\n"
    return body


def gh_api(args: list[str], payload: dict | None = None) -> str:
    command = ["gh", "api", *args]
    if payload is not None:
        command.extend(["--input", "-"])
    result = subprocess.run(
        command,
        input=None if payload is None else json.dumps(payload),
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        raise SystemExit(result.returncode)
    return result.stdout


def existing_comment_ids(repo: str, pr_number: str) -> list[int]:
    comments = json.loads(
        gh_api(["--paginate", f"repos/{repo}/issues/{pr_number}/comments"])
    )
    matches = [
        int(comment["id"])
        for comment in comments
        if (comment.get("user") or {}).get("login") == BOT_LOGIN
        and MARKER in (comment.get("body") or "")
    ]
    matches.sort()
    return matches


def upsert_comment(body: str) -> None:
    repo = os.environ["GITHUB_REPOSITORY"]
    pr_number = os.environ["PR_NUMBER"]
    matches = existing_comment_ids(repo, pr_number)
    if matches:
        gh_api(
            ["--method", "PATCH", f"repos/{repo}/issues/comments/{matches[0]}"],
            {"body": body},
        )
        for extra in matches[1:]:
            gh_api(["--method", "DELETE", f"repos/{repo}/issues/comments/{extra}"])
        print(f"Updated comment {matches[0]}")
        return
    created = json.loads(
        gh_api(
            ["--method", "POST", f"repos/{repo}/issues/{pr_number}/comments"],
            {"body": body},
        )
    )
    print(f"Created comment {created.get('id')}")


def main() -> None:
    test_log = read_log(Path("results/tests/test-results.txt"))
    benchmarks = Path("results/benchmarks")
    main_log = read_log(benchmarks / "bench-main.txt")
    pr_log = read_log(benchmarks / "bench-pr.txt")
    if main_log is None and pr_log is None:
        pr_log = read_log(benchmarks / "bench-results.txt")
    body = build_comment(
        test_log,
        main_log,
        pr_log,
        read_bench_status(benchmarks / "bench-status.txt"),
    )
    if "--print-only" in sys.argv:
        print(body)
        return
    upsert_comment(body)


if __name__ == "__main__":
    main()
