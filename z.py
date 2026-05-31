#!/usr/bin/env python3
"""
zip_source_certifier.py

A single-file, stdlib-only source/context archive builder for Rust workspaces,
AiDENs/Recall-style Codex handoffs, and research-heavy coding archives.

The goal is not merely to create a .zip. The goal is to create an archive that
can be inspected, audited, and trusted: included files are hashed, excluded files
are explained, required surfaces are checked, and common self-containment failures
are surfaced before the package leaves your machine.

Typical use:

  python3 zip_source_certifier.py \
    --root ~/Coding/Libraries/AiDENs \
    --profile aidens \
    --mode codex-context

  python3 zip_source_certifier.py \
    --root ~/Coding/Libraries \
    --profile libraries \
    --mode codex-context \
    --strict

Outputs by default:
  <archive>.zip
  <archive>.manifest.json
  <archive>.report.md
  <archive>.excluded.json
  <archive>.findings.json

Exit codes:
  0 = archive written / dry-run completed
  2 = validation failed under --strict
  1 = unexpected operational error
"""

from __future__ import annotations

import argparse
import fnmatch
import hashlib
import json
import os
import re
import shutil
import stat
import sys
import unicodedata
import zipfile
from collections import Counter
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Sequence

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback
    tomllib = None

SCRIPT_VERSION = "2026.05.22-p31"
UTC = timezone.utc
ZIP_EPOCH = (1980, 1, 1, 0, 0, 0)
PACKAGE_POLICY_SCHEMA_NAME = "PackagePolicyV1"

PROFILES = (
    "auto",
    "aidens",
    "libraries",
    "recall",
    "recall-coding",
    "semantic-memory",
    "generic-rust",
    "generic",
    "research",
)

MODES = (
    "source-clean",
    "release-context",
    "next-codex-context",
    "codex-context",
    "codex-run-full",
    "full-context",
    "research-context",
    "audit-full",
)

# Directories that are almost never useful in a source/context handoff and are
# dangerous/noisy enough to prune early.
ALWAYS_EXCLUDED_DIR_NAMES = {
    ".git",
    ".hg",
    ".svn",
    ".aicc-out",
    ".claude",
    ".cache",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".tox",
    ".venv",
    "venv",
    "env",
    "__pycache__",
    "target",
    "node_modules",
    "bower_components",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    ".svelte-kit",
    "coverage",
    ".tmp-rust",
    "tmp-rust",
    "library-source-zips",
    "source-zips",
    "zips",
    "rendered",
    "ARCHIVE",
    "archive",
}

EXCLUDED_DIR_PREFIXES = (
    ".venv",
    "tmp",
    "tmp-",
    "source-zips-",
    "target-",
    "venv",
)

GENERATED_SCHEMA_DIR_NAMES = {
    "schemas.generated",
    "generated-schemas",
    "schema.generated",
}

CODEX_ARTIFACT_DIR_NAMES = {
    ".codex",
    "codex",
}

EDITOR_CONFIG_DIR_NAMES = {
    ".idea",
    ".vscode",
}

ARCHIVE_EXTENSIONS = {
    ".zip",
    ".tar",
    ".gz",
    ".tgz",
    ".7z",
    ".rar",
    ".bz2",
    ".xz",
    ".zst",
}

BINARY_EXTENSIONS = {
    ".a",
    ".bin",
    ".class",
    ".dylib",
    ".dll",
    ".dmg",
    ".exe",
    ".jar",
    ".lib",
    ".o",
    ".obj",
    ".pdb",
    ".pyc",
    ".pyo",
    ".rlib",
    ".rmeta",
    ".so",
    ".wasm",
    ".woff",
    ".woff2",
}

DATABASE_EXTENSIONS = {
    ".db",
    ".sqlite",
    ".sqlite3",
    ".duckdb",
}

DOC_BINARY_EXTENSIONS = {
    ".pdf",
    ".docx",
    ".pptx",
    ".xlsx",
}

IMAGE_EXTENSIONS = {
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".webp",
    ".svg",
    ".ico",
}

LOG_EXTENSIONS = {
    ".log",
}

PATCH_EVIDENCE_EXTENSIONS = {
    ".diff",
    ".patch",
}

GENERATED_SIDECAR_SUFFIXES = (
    ".manifest.json",
    ".report.md",
    ".excluded.json",
    ".findings.json",
)

ALLOWED_TEXT_EXTENSIONS = {
    ".rs",
    ".toml",
    ".lock",
    ".md",
    ".markdown",
    ".txt",
    ".json",
    ".jsonl",
    ".ndjson",
    ".ron",
    ".yml",
    ".yaml",
    ".csv",
    ".tsv",
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".mjs",
    ".cjs",
    ".css",
    ".scss",
    ".html",
    ".htm",
    ".sql",
    ".sh",
    ".bash",
    ".zsh",
    ".ps1",
    ".py",
    ".pyi",
    ".proto",
    ".graphql",
    ".gql",
    ".schema",
    ".jsonschema",
    ".jinja",
    ".j2",
    ".tmpl",
    ".template",
    ".service",
    ".timer",
    ".conf",
    ".cfg",
    ".ini",
}

ALLOWED_BASENAMES = {
    ".dockerignore",
    ".editorconfig",
    ".gitattributes",
    ".gitignore",
    ".nvmrc",
    ".python-version",
    "AGENTS",
    "AUTHORS",
    "CHANGELOG",
    "CODEOWNERS",
    "CONTRIBUTING",
    "COPYING",
    "Containerfile",
    "Dockerfile",
    "Justfile",
    "LICENSE",
    "Makefile",
    "NOTICE",
    "Procfile",
    "py.typed",
    "README",
    "SECURITY",
    "rust-toolchain",
}

ALLOWED_BASENAME_PREFIXES = (
    "AUTHORS",
    "CHANGELOG",
    "COPYING",
    "LICENSE",
    "NOTICE",
    "README",
)

ALLOWED_ENV_SAMPLE_NAMES = {
    ".env.example",
    ".env.sample",
    ".env.template",
    "env.example",
    "env.sample",
    "env.template",
}

SECRETISH_FILENAMES = {
    ".env",
    ".env.local",
    ".env.production",
    ".env.development",
    ".settings.json",
    ".npmrc",
    ".pypirc",
    ".netrc",
    "id_rsa",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
}

SECRETISH_NAME_RE = re.compile(
    r"(^|[_.\-])(secret|secrets|credentials?|private[_\-]?key)([_.\-]|$)",
    re.IGNORECASE,
)

SECRETISH_EXTENSIONS = {
    ".pem",
    ".key",
    ".p12",
    ".pfx",
}

NAMED_SECRET_ASSIGNMENT_RE = re.compile(
    r"(?i)\b(?:AWS_SECRET_ACCESS_KEY|AWS_ACCESS_KEY_ID|OPENAI_API_KEY|ANTHROPIC_API_KEY|GITHUB_TOKEN|GH_TOKEN|PASSWORD|PASSWD|API[_-]?KEY|SECRET|TOKEN)\b\s*[:=]\s*['\"]?[A-Za-z0-9_./+=\-]{16,}"
)

RUST_FIELD_FORWARDING_SECRET_ASSIGNMENT_RE = re.compile(
    r"^(?:self|super|crate|[a-z_][A-Za-z0-9_]*)(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*)+(?:\s*\(\s*\))?(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*\s*(?:\(\s*\))?)*$"
)

# Conservative. This catches high-risk mistakes without trying to become a DLP tool.
SECRET_CONTENT_PATTERNS: list[tuple[str, re.Pattern[str], str]] = [
    (
        "private-key-block",
        re.compile(r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----"),
        "error",
    ),
    (
        "openai-like-key",
        re.compile(r"\bsk-[A-Za-z0-9_\-]{20,}\b"),
        "error",
    ),
    (
        "github-token",
        re.compile(r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9_]{20,}\b|\bgithub_pat_[A-Za-z0-9_]{20,}\b"),
        "error",
    ),
    (
        "slack-token",
        re.compile(r"\bxox[baprs]-[A-Za-z0-9\-]{20,}\b"),
        "error",
    ),
    (
        "named-secret-assignment",
        NAMED_SECRET_ASSIGNMENT_RE,
        "warning",
    ),
]

INCLUDE_LITERAL_RE = re.compile(
    r"include_(?:str|bytes)!\(\s*\"([^\"]+)\"\s*\)"
)

INCLUDE_CARGO_MANIFEST_RE = re.compile(
    r"include_(?:str|bytes)!\(\s*concat!\(\s*env!\(\s*\"CARGO_MANIFEST_DIR\"\s*\)\s*,\s*\"([^\"]+)\"",
    re.MULTILINE,
)

CARGO_PATH_DEP_RE = re.compile(r"\bpath\s*=\s*\"([^\"]+)\"")

CARGO_DEP_TABLE_NAMES = {
    "dependencies",
    "dev-dependencies",
    "build-dependencies",
}

THIRD_PARTY_SOURCE_DIR_NAMES = {
    "vendor",
    "third_party",
    "third-party",
    "external",
    "deps",
}

ADVISORY_RUST_REF_DIR_NAMES = {
    "benches",
    "examples",
    "fixtures",
    "reference",
    "test",
    "testdata",
    "tests",
}

PROJECT_SCRIPT_DIR_NAMES = {
    ".github",
    "bin",
    "ci",
    "script",
    "scripts",
    "tool",
    "tools",
}

SECRET_PLACEHOLDER_TOKENS = {
    "dummy",
    "example",
    "fake",
    "fixture",
    "invalid",
    "never-store",
    "placeholder",
    "redact",
    "sample",
    "test",
    "your-",
    "your_",
}

SCRIPT_REF_RES = [
    re.compile(r"(?:^|\s)(?:source|\.)\s+([A-Za-z0-9_./\-]+\.sh)(?:\s|$)"),
    re.compile(r"(?:^|\s)(?:python3?|bash|sh|zsh)\s+([A-Za-z0-9_./\-]+\.(?:py|sh|bash|zsh))(?:\s|$)"),
]

CODEX_ARCHIVE_MANIFEST_VERSION = "CodexRunArchiveManifestV1"
ROOT_MARKDOWN_ARCHIVE_MANIFEST_VERSION = "RootMarkdownArchiveManifestV1"
ROOT_PACKAGE_ARCHIVE_MANIFEST_VERSION = "RootPackageArtifactArchiveManifestV1"
CODEX_RUN_INDEX = "docs/codex-runs/CODEX_RUN_INDEX.md"
CODEX_CURRENT_RUN = "docs/codex-runs/CURRENT_RUN.md"
CODEX_ARCHIVAL_POLICY = "docs/codex-runs/ARCHIVAL_POLICY.md"
CODEX_ARTIFACT_CLASSIFICATION = "docs/codex-runs/CODEX_ARTIFACT_CLASSIFICATION.json"
ROOT_MARKDOWN_ARCHIVE_DIR = "docs/root-markdown-archive"
ROOT_MARKDOWN_ARCHIVE_MANIFEST = "ROOT_MARKDOWN_ARCHIVE_MANIFEST.json"
ROOT_PACKAGE_ARCHIVE_DIR = "docs/source-packages/archive"
ROOT_PACKAGE_ARCHIVE_MANIFEST = "PACKAGE_ARTIFACT_ARCHIVE_MANIFEST.json"
ROOT_PACKAGE_ARCHIVE_ROLES = {
    "next-codex-context",
    "codex-run-full",
    "audit-full",
}
CONTEXT_LOG_BASENAMES = {
    "commands_run.log",
    "commands_run.receipts.jsonl",
}
CONTEXT_COMMAND_EVIDENCE_BASENAMES = CONTEXT_LOG_BASENAMES | {
    "COMMAND_RECEIPTS.jsonl",
    "COMMAND_EXECUTION_RECEIPTS.jsonl",
}
CONTEXT_COMMAND_EVIDENCE_MARKDOWN_RE = re.compile(r"(^|/)[A-Za-z0-9_.-]*COMMANDS_RUN\.md$")
CONTEXT_COMMAND_RECEIPT_SYNTHETIC_PATH = ".zpy/COMMAND_RECEIPTS.jsonl"
CONTEXT_LOG_ROLES = {
    "next-codex-context",
    "codex-run-full",
    "audit-full",
}
PATCH_EVIDENCE_ROLES = {
    "next-codex-context",
    "codex-run-full",
    "research-context",
    "audit-full",
}
CONTEXT_REQUIRED_ROLES = {
    "next-codex-context",
    "codex-run-full",
    "audit-full",
}
CPG_RUN_ARTIFACT_DIRS = {
    ".cpg/runs",
    ".cpg/hook_receipts",
}
ROOT_MARKDOWN_PROTECTED_FILES = {
    "AGENTS.md",
    "CLAUDE.md",
    "README.md",
    "CONTRIBUTING.md",
    "LICENSE.md",
    "CHANGELOG.md",
    "SECURITY.md",
    "CODE_OF_CONDUCT.md",
    "SUPPORT.md",
    "SUPPORT_PROFILE.md",
    "SOURCE_BASIS.md",
    "STATUS.md",
    "ARCHITECTURE.md",
    "DESIGN.md",
    "ROADMAP.md",
    "SHADOW_SEMANTICS_AUDIT.md",
}
ROOT_MARKDOWN_CANDIDATE_PATTERNS = [
    "*AUDIT*.MD",
    "*HARD_AUDIT*.MD",
    "*ISSUE_MATRIX*.MD",
    "*RISK_REGISTER*.MD",
    "*PROMPT*.MD",
    "*MASTER*.MD",
    "*SNAPSHOT*.MD",
    "*STATUS_DASHBOARD*.MD",
    "*IMPLEMENTATION_PLAYBOOK*.MD",
    "*CONFORMANCE*.MD",
    "*HARDENING*.MD",
    "*PLAN*.MD",
    "*TENSOR*.MD",
    "*MATRIX*.MD",
]
ROOT_MARKDOWN_PROTECTED_FILES_UPPER = {name.upper() for name in ROOT_MARKDOWN_PROTECTED_FILES}
ROOT_PACKAGE_PROTECTED_FILE_PATTERNS = {
    "AGENTS.md",
    "Cargo.lock",
    "Cargo.toml",
    "pyproject.toml",
    "README.md",
    "z.py",
}
ROOT_PACKAGE_PROTECTED_PREFIXES = (
    "CHANGELOG",
    "CONTRIBUTING",
    "LICENSE",
    "SECURITY",
)
ROOT_PACKAGE_ARTIFACT_PATTERNS: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"^.*-next-codex-context-[A-Za-z0-9T]+Z?\.zip$"), "prior-context-archive"),
    (re.compile(r"^.*-next-codex-context-[A-Za-z0-9T]+Z?\.(?:manifest\.json|report\.md|excluded\.json|findings\.json)$"), "prior-generated-sidecar"),
    (re.compile(r"^.*-next-codex-context-[A-Za-z0-9T]+Z?\.codex-archive\.json$"), "prior-codex-archive-report"),
    (re.compile(r"^.*\.codex-archive\.json$"), "prior-codex-archive-report"),
    (re.compile(r"^README_BUNDLE\.md$"), "prior-bundle-readme"),
    (re.compile(r"^BUNDLE_MANIFEST\.json$"), "prior-bundle-manifest"),
    (re.compile(r"^.*_BUNDLE\.md$"), "prior-bundle-doc"),
    (re.compile(r"^.*_PROMPT\.md$"), "root-prompt-residue"),
    (re.compile(r"^.*_AUDIT\.md$"), "root-audit-residue"),
    (re.compile(r"^.*_ISSUE_MATRIX\.md$"), "root-issue-matrix-residue"),
    (re.compile(r"^.*_RISK_REGISTER\.md$"), "root-risk-register-residue"),
]

WINDOWS_RESERVED_BASENAMES = {
    "CON",
    "PRN",
    "AUX",
    "NUL",
    *(f"COM{i}" for i in range(1, 10)),
    *(f"LPT{i}" for i in range(1, 10)),
}

POLICY_TOP_LEVEL_KEYS = {
    "schema",
    "package",
    "modes",
    "protected_root_files",
    "required_files",
    "allowed_extensions",
    "allowed_basenames",
    "path_rules",
    "ecosystem_parity",
    "root_hygiene",
    "security",
    "archive",
    "provenance",
    "sbom",
}

POLICY_MODE_CONTEXTS = {
    "context": {"next-codex-context", "codex-context", "codex-run-full", "full-context", "research-context", "audit-full"},
    "audit": {"audit-full", "codex-run-full"},
    "release": {"source-clean", "release-context"},
}

PROTECTED_CODEX_ACTIVE_FILES = {
    "AGENTS.md",
    "Cargo.lock",
    "Cargo.toml",
    "Makefile",
    "README.md",
    "SOURCE_BASIS.md",
    "STATUS.md",
    "rust-toolchain.toml",
    "z.py",
}

CODEX_RUN_SEGMENT_RE = re.compile(r"^(?:p|P)(\d{1,3})(?:[_-]?(\d+))?$")
CODEX_RUN_PREFIX_RE = re.compile(r"^(?:p|P)(\d{1,3})(?:[_-]?(\d+))?(?:[_-]?([A-Z]\w*))?")
CODEX_ROOT_RUN_PREFIX_RE = re.compile(r"(?:^|[_\-/])(?:p|P)(\d{1,3})(?:[_-]?(\d+))?")
CODEX_CONTRACT_OWNERSHIP_PHASE_RE = re.compile(r"(?:^|/)\.codex_evidence/contract_ownership/(\d{2})(?:/|$)")
CODEX_RUN_MARKER_RE = re.compile(r"(?:^|[/_.-])(?:p|P)(\d{1,3})(?:[_-]?(\d+))?(?=$|[/_.-])")

CODEX_STALE_PATH_PATTERNS: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"^\.codex/"), "stale-codex-control"),
    (re.compile(r"^\.codex_evidence/"), "stale-codex-evidence"),
    (re.compile(r"^\.?CODEX_[^/]*(?:/|$)"), "stale-root-codex-control"),
    (re.compile(r"^\.?NEXT_CODEX_[^/]*(?:/|$)"), "stale-root-codex-control"),
    (re.compile(r"^CODEX_PROMPTS/"), "stale-codex-prompt-dir"),
    (re.compile(r"^.*_CODEX_RUN_PROMPT\.md(?:\..*)?$"), "stale-codex-run-prompt"),
    (re.compile(r"^docs/[Pp]\d"), "stale-run-doc"),
    (re.compile(r"^prompts/[Pp]\d"), "stale-run-prompt"),
    (re.compile(r"^prompts/p\d"), "stale-run-prompt"),
    (re.compile(r"^prompts/phase_injections/"), "stale-phase-injection-prompt"),
    (re.compile(r"^prompts/phases/"), "stale-phase-prompt"),
    (re.compile(r"^handoffs/[Pp]\d"), "stale-run-handoff"),
    (re.compile(r"^handoffs/p\d"), "stale-run-handoff"),
    (re.compile(r"^tasks/[Pp]\d"), "stale-run-task"),
    (re.compile(r"^tasks/p\d"), "stale-run-task"),
    (re.compile(r"^scripts/[Pp]\d+(?:[_-]?\d+)?[_-]"), "stale-run-script"),
    (re.compile(r"^scripts/assert_[Pp]\d+(?:[_-]?\d+)?[_-]"), "stale-run-script"),
    (re.compile(r"^install_[Pp]\d+(?:[_-]?\d+)?_overlay\.sh$"), "stale-run-install-script"),
]


@dataclass(frozen=True)
class Finding:
    code: str
    severity: str
    path: str
    detail: str


@dataclass(frozen=True)
class FileEntry:
    path: str
    bytes: int
    sha256: str
    mode: str
    executable: bool
    mtime_utc: str


@dataclass(frozen=True)
class SyntheticFile:
    path: str
    data: bytes
    mode: int = 0o644


@dataclass(frozen=True)
class ExcludedEntry:
    path: str
    reason: str


@dataclass(frozen=True)
class PrunedDirEntry:
    path: str
    reason: str


@dataclass(frozen=True)
class DecisionEntry:
    path: str
    decision: str
    reason: str
    source: str
    mode: str


@dataclass(frozen=True)
class EcosystemAdapterResult:
    ecosystem: str
    detected: bool
    manifests: list[str]
    dry_run_available: bool
    dry_run_command: str | None
    dry_run_status: str
    expected_files: list[str]
    missing_from_zpy_package: list[str]
    extra_in_zpy_package: list[str]
    findings: list[dict[str, str]]


@dataclass
class ArchiveReport:
    script: str
    script_version: str
    created_utc: str
    root: str
    archive_root: str
    include_roots: list[str]
    external_path_dep_roots: list[str]
    output: str
    profile_requested: str
    profile_resolved: str
    mode: str
    package_role: str
    strict: bool
    dry_run: bool
    deterministic_zip_timestamps: bool
    included_count: int
    included_bytes: int
    excluded_file_count: int
    pruned_dir_count: int
    findings_count: int
    error_count: int
    warning_count: int
    archive_sha256: str | None
    archive_zip_byte_sha256: str | None
    archive_sha256_semantics: str
    content_manifest_sha256: str | None
    archive_written: bool
    manifest_path: str | None
    report_path: str | None
    excluded_path: str | None
    findings_path: str | None
    decision_log_path: str | None
    policy_path: str | None
    ecosystem_parity: list[dict[str, Any]]
    codex_archive: dict[str, Any] | None
    root_markdown_archive: dict[str, Any] | None
    root_package_archive: dict[str, Any] | None


@dataclass(frozen=True)
class Policy:
    policy_path: str | None
    policy_document: dict[str, Any] | None
    profile: str
    mode: str
    package_role: str
    codex_current_run: str
    codex_artifact_classification: dict[str, str]
    include_external_path_deps: bool
    include_generated_schemas: bool
    include_codex_artifacts: bool
    include_codex_archive: bool
    include_root_markdown_archive: bool
    root_markdown_archive_root: str
    root_markdown_archive_root_rel: str
    include_root_package_archive: bool
    root_package_archive_root: str
    root_package_archive_root_rel: str
    include_editor_config: bool
    include_doc_binaries: bool
    include_images: bool
    include_logs: bool
    include_patch_artifacts: bool
    allow_secret_like_names: bool
    follow_symlinks: bool
    max_file_size_bytes: int
    secret_scan_max_bytes: int
    required_files: list[dict[str, Any]]
    path_rules: list[dict[str, Any]]
    allowed_extensions: list[str]
    allowed_basenames: list[str]
    ecosystem_parity_enabled: bool
    ecosystem_parity_default_severity: str
    ecosystem_parity_adapters: dict[str, str]
    fail_on_unicode_collision: bool
    fail_on_case_collision: bool
    fail_on_windows_reserved_name: bool
    emit_decision_log: bool
    source_date_epoch: int | None


@dataclass
class BuildResult:
    report: ArchiveReport
    files: list[FileEntry]
    excluded: list[ExcludedEntry]
    pruned_dirs: list[PrunedDirEntry]
    findings: list[Finding]
    decisions: list[DecisionEntry]
    ecosystem_parity: list[EcosystemAdapterResult]


@dataclass(frozen=True)
class CodexArchiveCandidate:
    original_path: str
    run_id: str
    reason: str
    sha256: str
    bytes: int
    mtime_utc: str


@dataclass
class CodexArchiveResult:
    enabled: bool
    dry_run: bool
    verify_only: bool
    archive_only: bool
    current_run: str
    archive_root: str
    report_path: str | None
    stale_active_before: list[str]
    planned: list[dict[str, Any]]
    moved: list[dict[str, Any]]
    skipped_existing: list[dict[str, Any]]
    collisions: list[dict[str, Any]]
    unclassified: list[dict[str, Any]]
    active_stale_after: list[str]
    manifest_paths: list[str]
    errors: list[str]


@dataclass
class RootMarkdownArchiveResult:
    enabled: bool
    dry_run: bool
    verify_only: bool
    archive_only: bool
    current_run: str
    archive_root: str
    archive_dir: str
    manifest_path: str | None
    inspected_count: int
    protected_count: int
    candidate_count: int
    ambiguous_count: int
    planned_count: int
    moved_count: int
    skipped_existing_count: int
    collision_count: int
    manifest_written: bool
    candidate_paths: list[str]
    protected_paths: list[str]
    ambiguous_paths: list[str]
    collisions: list[dict[str, Any]]
    errors: list[str]


@dataclass
class RootPackageArchiveResult:
    enabled: bool
    dry_run: bool
    verify_only: bool
    archive_only: bool
    archive_root: str
    archive_dir: str
    manifest_path: str | None
    inspected_count: int
    protected_count: int
    candidate_count: int
    planned_count: int
    moved_count: int
    skipped_existing_count: int
    collision_count: int
    manifest_written: bool
    candidate_paths: list[str]
    protected_paths: list[str]
    moved: list[dict[str, Any]]
    skipped_existing: list[dict[str, Any]]
    collisions: list[dict[str, Any]]
    errors: list[str]


def utc_now_iso() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def to_posix(path: Path | str) -> str:
    return str(path).replace(os.sep, "/")


def is_relative_to(child: Path, parent: Path) -> bool:
    try:
        child.resolve().relative_to(parent.resolve())
        return True
    except ValueError:
        return False


def safe_relative(path: Path, root: Path) -> Path:
    try:
        return path.resolve().relative_to(root.resolve())
    except ValueError:
        # Symlink entries can resolve outside the workspace even though the
        # directory entry itself is under root. Keep relative names lexical;
        # target containment is checked separately where symlinks are allowed.
        return Path(os.path.abspath(path)).relative_to(Path(os.path.abspath(root)))


def read_text_lossy(path: Path, limit_bytes: int | None = None) -> str | None:
    try:
        if limit_bytes is None:
            data = path.read_bytes()
        else:
            with path.open("rb") as f:
                data = f.read(limit_bytes)
    except OSError:
        return None
    if b"\x00" in data[:4096]:
        return None
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def text_file_policy_reason(path: Path, limit_bytes: int | None = None) -> str | None:
    try:
        if limit_bytes is None:
            data = path.read_bytes()
        else:
            with path.open("rb") as f:
                data = f.read(limit_bytes)
    except OSError:
        return "read-failed"
    if b"\x00" in data[:4096]:
        return "binary-null-byte"
    try:
        data.decode("utf-8")
    except UnicodeDecodeError:
        return "non-utf8-text-file"
    return None


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_json_payload(payload: object) -> str:
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def mode_string(path: Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):06o}"


def is_executable(path: Path) -> bool:
    return bool(stat.S_IMODE(path.stat().st_mode) & 0o111)


def file_mtime_utc(path: Path) -> str:
    return datetime.fromtimestamp(path.stat().st_mtime, UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def codex_run_stamp() -> str:
    return datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")


def normalize_codex_run_id(value: str | None) -> str:
    if not value:
        return "unclassified"
    cleaned = value.strip().replace("-", "_").replace("/", "_")
    match = CODEX_RUN_PREFIX_RE.match(cleaned)
    if match:
        major = int(match.group(1))
        numeric_minor = match.group(2)
        letter_suffix = match.group(3)
        parts = [f"P{major}"]
        if numeric_minor:
            parts.append(numeric_minor)
        if letter_suffix:
            parts.append(letter_suffix)
        if len(parts) > 1:
            return "_".join(parts)
        return parts[0]
    if cleaned.startswith("legacy_"):
        return cleaned.replace("_", "-")
    return cleaned.upper()


def package_role_for_mode(mode: str) -> str:
    if mode == "codex-context":
        return "next-codex-context"
    if mode == "full-context":
        return "codex-run-full"
    return mode


def current_run_tokens(current_run: str) -> set[str]:
    current = normalize_codex_run_id(current_run)
    tokens = {current, current.lower(), current.replace("_", "-"), current.lower().replace("_", "-")}
    return {token for token in tokens if token}


def path_has_current_run_marker(rel: str, current_run: str) -> bool:
    tokens = current_run_tokens(current_run)
    parts = Path(rel).parts
    for part in parts:
        stripped = part.strip()
        lower = stripped.lower()
        stem = Path(stripped).stem.lower()
        if lower in {token.lower() for token in tokens} or stem in {token.lower() for token in tokens}:
            return True
        if any(
            lower.startswith(f"{token.lower()}_")
            or lower.startswith(f"{token.lower()}-")
            or lower.startswith(f"{token.lower()}.")
            for token in tokens
        ):
            return True
    current = normalize_codex_run_id(current_run)
    for match in CODEX_RUN_MARKER_RE.finditer(rel):
        major = int(match.group(1))
        minor = match.group(2)
        marker = f"P{major}" + (f"_{int(minor)}" if minor else "")
        if normalize_codex_run_id(marker) == current:
            return True
    return False


def path_has_noncurrent_run_marker(rel: str, current_run: str) -> bool:
    current = normalize_codex_run_id(current_run)
    for match in CODEX_RUN_MARKER_RE.finditer(rel):
        major = int(match.group(1))
        minor = match.group(2)
        marker = f"P{major}" + (f"_{int(minor)}" if minor else "")
        if normalize_codex_run_id(marker) != current:
            return True
    return False


def active_run_surface(rel: str) -> bool:
    parts = Path(rel).parts
    if not parts:
        return False
    top = parts[0]
    if top in {"audit", "evals", "fixtures", "handoffs", "prompts", "repo_overlay", "scripts", "supporting", "tasks", "templates"}:
        return True
    return top == "docs" and not rel.startswith("docs/codex-runs/archive/")


def load_codex_artifact_classification(root: Path) -> dict[str, str]:
    path = root / CODEX_ARTIFACT_CLASSIFICATION
    if not path.exists():
        return {}
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    entries = payload.get("artifacts", payload if isinstance(payload, list) else [])
    classification: dict[str, str] = {}
    if isinstance(entries, list):
        for item in entries:
            if not isinstance(item, dict):
                continue
            rel = str(item.get("path", "")).strip("/")
            kind = str(item.get("classification", item.get("class", ""))).strip()
            if rel and kind:
                classification[rel] = kind
    return classification


def codex_rel_variants(rel: str) -> list[str]:
    rel = rel.strip("/")
    parts = rel.split("/") if rel else []
    variants = [rel]
    anchor_names = {
        ".codex",
        ".codex_evidence",
        "CODEX_PROMPTS",
        "docs",
        "handoffs",
        "prompts",
        "scripts",
        "tasks",
    }
    for idx, part in enumerate(parts):
        if part in anchor_names or part.startswith("CODEX_") or part.startswith("NEXT_CODEX_"):
            suffix = "/".join(parts[idx:])
            if suffix not in variants:
                variants.append(suffix)
    return variants


def is_codex_archive_rel(rel: str) -> bool:
    return any(variant.startswith("docs/codex-runs/archive/") for variant in codex_rel_variants(rel))


def is_codex_archive_dir_rel(rel: str) -> bool:
    return any(
        variant == "docs/codex-runs/archive" or variant.startswith("docs/codex-runs/archive/")
        for variant in codex_rel_variants(rel)
    )


def is_cpg_run_artifact_dir_rel(rel: str) -> bool:
    rel = rel.strip("/")
    return any(rel == artifact_dir or rel.startswith(f"{artifact_dir}/") for artifact_dir in CPG_RUN_ARTIFACT_DIRS)


def is_allowed_current_codex_rel(rel: str, current_run: str) -> bool:
    current = normalize_codex_run_id(current_run)
    current_phase_match = re.search(r"(?:^|_)p(\d+)(?:_|$)", current, flags=re.IGNORECASE)
    current_phase_prefix = f"p{current_phase_match.group(1)}_" if current_phase_match else ""
    for variant in codex_rel_variants(rel):
        if variant in PROTECTED_CODEX_ACTIVE_FILES:
            return True
        if variant in {CODEX_RUN_INDEX, CODEX_CURRENT_RUN, CODEX_ARCHIVAL_POLICY, CODEX_ARTIFACT_CLASSIFICATION}:
            return True
        if variant.startswith("docs/codex-runs/") and not variant.startswith("docs/codex-runs/archive/"):
            return True
        if current_phase_prefix and variant.startswith(f"scripts/{current_phase_prefix}"):
            return True
        if path_has_current_run_marker(variant, current):
            return True
    return False


def stale_codex_reason_for_rel(
    rel: str,
    current_run: str,
    classification: dict[str, str] | None = None,
) -> str | None:
    classification = classification or {}
    rel = rel.strip("/")
    classified_as = classification.get(rel)
    active_classifications = {
        "active-regression-fixture",
        "active-support-matrix",
        "active-operator-doc",
        "compatibility-fixture",
        "current-instruction",
        "current-run-evidence",
    }
    if classified_as in active_classifications:
        return None
    for variant in codex_rel_variants(rel):
        if is_allowed_current_codex_rel(variant, current_run):
            return None
        if is_codex_archive_rel(variant):
            return None
        for regex, reason in CODEX_STALE_PATH_PATTERNS:
            if regex.search(variant):
                return reason
        if active_run_surface(variant) and path_has_noncurrent_run_marker(variant, current_run):
            return "stale-run-marked-artifact"
    return None


def infer_codex_run_id(rel: str, reason: str) -> str:
    for variant in codex_rel_variants(rel):
        contract_match = CODEX_CONTRACT_OWNERSHIP_PHASE_RE.search(variant)
        if contract_match:
            return f"legacy-contract-ownership-{contract_match.group(1)}"

        for part in Path(variant).parts:
            match = CODEX_RUN_SEGMENT_RE.match(part)
            if match:
                major = int(match.group(1))
                minor = match.group(2)
                return f"P{major}" + (f"_{int(minor)}" if minor else "")

        name_match = CODEX_ROOT_RUN_PREFIX_RE.search(Path(variant).name)
        if name_match:
            major = int(name_match.group(1))
            minor = name_match.group(2)
            return f"P{major}" + (f"_{int(minor)}" if minor else "")

    if reason in {"stale-phase-injection-prompt", "stale-phase-prompt", "stale-root-codex-control", "stale-codex-prompt-dir"}:
        return "unclassified"
    return "unclassified"


def safe_archive_component(value: str) -> str:
    return re.sub(r"[^A-Za-z0-9_.-]+", "_", value).strip("._") or "unclassified"


def infer_profile(root: Path) -> str:
    name = root.name.lower()
    if name == "semantic-memory":
        return "semantic-memory"
    if "aidens" in name:
        return "aidens"
    if "recall-coding" in name or "recall_coding" in name:
        return "recall-coding"
    if name == "recall" or name.startswith("recall-"):
        return "recall"
    if name in {"libraries", "library", "libs"}:
        return "libraries"
    if (root / "Cargo.toml").exists():
        return "generic-rust"
    md_files = list(root.glob("*.md"))
    if md_files and not (root / "src").exists():
        return "research"
    return "generic"


def read_policy_document(path: Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8")
    if path.suffix.lower() == ".json":
        payload = json.loads(text)
    else:
        if tomllib is None:
            raise ValueError("TOML policy files require Python 3.11+ tomllib support")
        payload = tomllib.loads(text)
    if not isinstance(payload, dict):
        raise ValueError("PackagePolicyV1 policy must be a table/object")
    return payload


def validate_policy_document(payload: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    extra = sorted(set(payload) - POLICY_TOP_LEVEL_KEYS)
    if extra:
        errors.append(f"unknown top-level policy keys: {', '.join(extra)}")
    if payload.get("schema") != PACKAGE_POLICY_SCHEMA_NAME:
        errors.append(f"schema must be {PACKAGE_POLICY_SCHEMA_NAME!r}")
    package = payload.get("package")
    if not isinstance(package, dict):
        errors.append("package must be an object/table")
    elif not isinstance(package.get("name"), str) or not package.get("name", "").strip():
        errors.append("package.name is required")
    modes = payload.get("modes")
    if not isinstance(modes, dict):
        errors.append("modes must be an object/table")
    for key in ("protected_root_files", "allowed_extensions", "allowed_basenames"):
        value = payload.get(key)
        if value is not None and not (isinstance(value, list) and all(isinstance(item, str) for item in value)):
            errors.append(f"{key} must be a string array")
    required = payload.get("required_files", [])
    if required is not None:
        if not isinstance(required, list):
            errors.append("required_files must be an array")
        else:
            for idx, item in enumerate(required):
                if not isinstance(item, dict) or not isinstance(item.get("path"), str):
                    errors.append(f"required_files[{idx}] must include string path")
                if isinstance(item, dict) and item.get("mode", "all") not in {"all", "context", "audit", "release"}:
                    errors.append(f"required_files[{idx}].mode is invalid")
    path_rules = payload.get("path_rules", [])
    if path_rules is not None:
        if not isinstance(path_rules, list):
            errors.append("path_rules must be an array")
        else:
            for idx, item in enumerate(path_rules):
                if not isinstance(item, dict):
                    errors.append(f"path_rules[{idx}] must be an object")
                    continue
                if not isinstance(item.get("pattern"), str):
                    errors.append(f"path_rules[{idx}].pattern is required")
                if item.get("decision") not in {"include", "exclude", "quarantine", "archive-root"}:
                    errors.append(f"path_rules[{idx}].decision is invalid")
    return errors


def load_policy_file(value: str | None) -> tuple[Path | None, dict[str, Any] | None, list[Finding]]:
    if not value:
        return None, None, []
    path = Path(value).expanduser().resolve()
    findings: list[Finding] = []
    try:
        payload = read_policy_document(path)
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        return path, None, [Finding(
            code="package-policy-load-failed",
            severity="error",
            path=str(path),
            detail=str(exc),
        )]
    for error in validate_policy_document(payload):
        findings.append(Finding(
            code="package-policy-invalid",
            severity="error",
            path=str(path),
            detail=error,
        ))
    return path, payload, findings


def policy_mode_section(policy_doc: dict[str, Any] | None, mode: str, package_role: str) -> dict[str, Any]:
    if not policy_doc:
        return {}
    modes = policy_doc.get("modes", {})
    if not isinstance(modes, dict):
        return {}
    for key in (mode, package_role):
        value = modes.get(key)
        if isinstance(value, dict):
            return value
    return {}


def policy_bool(section: dict[str, Any], key: str, fallback: bool) -> bool:
    value = section.get(key)
    return value if isinstance(value, bool) else fallback


def source_date_epoch_from_value(value: Any) -> int | None:
    if value is None:
        env_value = os.environ.get("SOURCE_DATE_EPOCH")
        if env_value is None:
            return None
        value = env_value
    if isinstance(value, int):
        return value if value >= 0 else None
    if isinstance(value, str) and value.strip().isdigit():
        return int(value.strip())
    return None


def mode_matches_policy_rule(rule: dict[str, Any], mode: str, package_role: str) -> bool:
    modes = rule.get("modes")
    if modes is None:
        return True
    if not isinstance(modes, list):
        return False
    return mode in modes or package_role in modes


def required_file_applies(required: dict[str, Any], mode: str, package_role: str) -> bool:
    required_mode = str(required.get("mode", "all"))
    if required_mode == "all":
        return True
    if required_mode in POLICY_MODE_CONTEXTS:
        return mode in POLICY_MODE_CONTEXTS[required_mode] or package_role in POLICY_MODE_CONTEXTS[required_mode]
    return required_mode in {mode, package_role}


def make_policy(args: argparse.Namespace, root: Path, resolved_profile: str) -> Policy:
    mode = args.mode
    package_role = package_role_for_mode(mode)
    policy_path, policy_doc, _policy_findings = load_policy_file(getattr(args, "policy", None))
    mode_policy = policy_mode_section(policy_doc, mode, package_role)
    security_policy = policy_doc.get("security", {}) if policy_doc else {}
    if not isinstance(security_policy, dict):
        security_policy = {}
    archive_policy = policy_doc.get("archive", {}) if policy_doc else {}
    if not isinstance(archive_policy, dict):
        archive_policy = {}
    ecosystem_policy = policy_doc.get("ecosystem_parity", {}) if policy_doc else {}
    if not isinstance(ecosystem_policy, dict):
        ecosystem_policy = {}
    include_generated_schemas = (
        args.include_generated_schemas
        if args.include_generated_schemas is not None
        else package_role in {"next-codex-context", "codex-run-full"}
    )
    include_codex_artifacts = (
        args.include_codex_artifacts
        if args.include_codex_artifacts is not None
        else False
    )
    include_codex_archive = bool(args.include_codex_archive or package_role == "audit-full")
    include_root_markdown_archive = bool(args.include_root_markdown_archive)
    include_root_package_archive = bool(args.include_root_package_archive or package_role == "audit-full")
    root_markdown_archive_root = resolve_root_markdown_archive_root(root, args.root_markdown_archive_root)
    try:
        root_markdown_archive_root_rel = to_posix(safe_relative(root_markdown_archive_root, root))
    except ValueError:
        root_markdown_archive_root_rel = to_posix(root_markdown_archive_root)
    root_package_archive_root = resolve_root_package_archive_root(root, args.root_package_archive_root)
    try:
        root_package_archive_root_rel = to_posix(safe_relative(root_package_archive_root, root))
    except ValueError:
        root_package_archive_root_rel = to_posix(root_package_archive_root)
    include_doc_binaries = (
        args.include_doc_binaries
        if args.include_doc_binaries is not None
        else policy_bool(mode_policy, "include_doc_binaries", package_role in {"research-context", "codex-run-full", "audit-full"})
    )
    include_images = (
        args.include_images
        if args.include_images is not None
        else policy_bool(mode_policy, "include_images", package_role in {"research-context", "codex-run-full", "audit-full"})
    )
    include_logs = (
        args.include_logs
        if args.include_logs is not None
        else policy_bool(mode_policy, "include_logs", False)
    )
    include_patch_artifacts = policy_bool(
        mode_policy,
        "include_patch_artifacts",
        package_role in PATCH_EVIDENCE_ROLES,
    )
    max_file_size_bytes = (
        int(security_policy["max_file_size_bytes"])
        if isinstance(security_policy.get("max_file_size_bytes"), int)
        else int(args.max_file_size_mb * 1024 * 1024) if args.max_file_size_mb > 0 else 0
    )
    adapters = ecosystem_policy.get("adapters", {})
    if not isinstance(adapters, dict):
        adapters = {}
    required_files = policy_doc.get("required_files", []) if policy_doc else []
    path_rules = policy_doc.get("path_rules", []) if policy_doc else []
    allowed_extensions = policy_doc.get("allowed_extensions", []) if policy_doc else []
    allowed_basenames = policy_doc.get("allowed_basenames", []) if policy_doc else []
    return Policy(
        policy_path=str(policy_path) if policy_path else None,
        policy_document=policy_doc,
        profile=resolved_profile,
        mode=mode,
        package_role=package_role,
        codex_current_run=normalize_codex_run_id(args.codex_current_run),
        codex_artifact_classification=load_codex_artifact_classification(root),
        include_external_path_deps=args.include_external_path_deps or resolved_profile == "semantic-memory",
        include_generated_schemas=include_generated_schemas,
        include_codex_artifacts=include_codex_artifacts,
        include_codex_archive=include_codex_archive,
        include_root_markdown_archive=include_root_markdown_archive,
        root_markdown_archive_root=str(root_markdown_archive_root),
        root_markdown_archive_root_rel=root_markdown_archive_root_rel,
        include_root_package_archive=include_root_package_archive,
        root_package_archive_root=str(root_package_archive_root),
        root_package_archive_root_rel=root_package_archive_root_rel,
        include_editor_config=args.include_editor_config,
        include_doc_binaries=include_doc_binaries,
        include_images=include_images,
        include_logs=include_logs,
        include_patch_artifacts=include_patch_artifacts,
        allow_secret_like_names=args.allow_secret_like_names or policy_bool(security_policy, "allow_secret_like_names", False),
        follow_symlinks=args.follow_symlinks or policy_bool(security_policy, "follow_symlinks", False),
        max_file_size_bytes=max_file_size_bytes,
        secret_scan_max_bytes=int(args.secret_scan_max_kb * 1024),
        required_files=[item for item in required_files if isinstance(item, dict)],
        path_rules=[item for item in path_rules if isinstance(item, dict)],
        allowed_extensions=[item for item in allowed_extensions if isinstance(item, str)],
        allowed_basenames=[item for item in allowed_basenames if isinstance(item, str)],
        ecosystem_parity_enabled=policy_bool(ecosystem_policy, "enabled", True),
        ecosystem_parity_default_severity=str(ecosystem_policy.get("default_severity", "info")),
        ecosystem_parity_adapters={str(k): str(v) for k, v in adapters.items()},
        fail_on_unicode_collision=policy_bool(security_policy, "fail_on_unicode_collision", True),
        fail_on_case_collision=policy_bool(security_policy, "fail_on_case_collision", package_role in {"source-clean", "release-context"}),
        fail_on_windows_reserved_name=policy_bool(security_policy, "fail_on_windows_reserved_name", True),
        emit_decision_log=policy_bool(archive_policy, "emit_decision_log", False),
        source_date_epoch=source_date_epoch_from_value(archive_policy.get("source_date_epoch")),
    )


def should_prune_dir(rel_dir: Path, dirname: str, policy: Policy) -> str | None:
    lower = dirname.lower()
    rel_posix = to_posix(rel_dir)
    archive_rel = policy.root_markdown_archive_root_rel.strip("/")
    if archive_rel and (rel_posix == archive_rel or rel_posix.startswith(f"{archive_rel}/")):
        if policy.include_root_markdown_archive:
            return None
        return "root-markdown-archive-disabled"
    package_archive_rel = policy.root_package_archive_root_rel.strip("/")
    if package_archive_rel and (rel_posix == package_archive_rel or rel_posix.startswith(f"{package_archive_rel}/")):
        if policy.include_root_package_archive:
            return None
        return "root-package-archive-disabled"
    if is_codex_archive_dir_rel(rel_posix):
        if policy.include_codex_archive:
            return None
        return "codex-archive-disabled"
    if policy.package_role == "next-codex-context" and is_cpg_run_artifact_dir_rel(rel_posix):
        return "cpg-run-artifacts-disabled"
    if dirname in ALWAYS_EXCLUDED_DIR_NAMES or lower in {d.lower() for d in ALWAYS_EXCLUDED_DIR_NAMES}:
        return "excluded-directory"
    if any(lower.startswith(prefix.lower()) for prefix in EXCLUDED_DIR_PREFIXES):
        return "excluded-directory-prefix"
    if lower in {d.lower() for d in GENERATED_SCHEMA_DIR_NAMES} and not policy.include_generated_schemas:
        return "generated-schemas-disabled"
    if lower in {d.lower() for d in CODEX_ARTIFACT_DIR_NAMES} and not policy.include_codex_artifacts:
        return "codex-artifacts-disabled"
    if lower in {d.lower() for d in EDITOR_CONFIG_DIR_NAMES} and not policy.include_editor_config:
        return "editor-config-disabled"
    return None


def is_secret_like_path(path: Path) -> bool:
    lower_name = path.name.lower()
    if lower_name in {
        "phase_16_config_environment_secrets_and_redaction.md",
    }:
        return False
    if lower_name in ALLOWED_ENV_SAMPLE_NAMES:
        return False
    if lower_name in SECRETISH_FILENAMES:
        return True
    if lower_name.startswith(".env.") and lower_name not in ALLOWED_ENV_SAMPLE_NAMES:
        return True
    if path.suffix.lower() in SECRETISH_EXTENSIONS:
        return True
    if SECRETISH_NAME_RE.search(lower_name):
        return True
    return False


def is_generated_sidecar_path(path: Path) -> bool:
    return any(path.name.endswith(suffix) for suffix in GENERATED_SIDECAR_SUFFIXES)


def is_fixture_sidecar_rel(rel: str) -> bool:
    rel = rel.strip("/")
    return rel.startswith("tests/fixtures/") and any(rel.endswith(suffix) for suffix in GENERATED_SIDECAR_SUFFIXES)


def is_context_receipt_log(rel: str, package_role: str) -> bool:
    p = Path(rel)
    return (
        package_role in CONTEXT_LOG_ROLES
        and p.name in CONTEXT_LOG_BASENAMES
    )


def is_context_command_evidence_rel(rel: str) -> bool:
    rel = rel.strip("/")
    if not rel:
        return False
    name = Path(rel).name
    return name in CONTEXT_COMMAND_EVIDENCE_BASENAMES or bool(CONTEXT_COMMAND_EVIDENCE_MARKDOWN_RE.search(rel))


def allowed_basename(path: Path) -> bool:
    name = path.name
    if name in ALLOWED_BASENAMES:
        return True
    upper = name.upper()
    return any(upper == p or upper.startswith(p + ".") or upper.startswith(p + "-") for p in ALLOWED_BASENAME_PREFIXES)


def decision_source_for_reason(reason: str) -> str:
    if reason.startswith("policy-"):
        return "package-policy"
    if reason.startswith("ecosystem-"):
        return "ecosystem-adapter"
    if "secret" in reason:
        return "security-gate"
    if "symlink" in reason or reason in {"special-file", "hardlink-detected"}:
        return "portability-gate"
    if reason in {"included-context-receipt-log", "included-patch-evidence"}:
        return "audit-evidence-policy"
    if reason.endswith("-disabled") or reason in {"archive-file", "binary-build-artifact", "database-file"}:
        return "hygiene-policy"
    if reason.startswith("included-"):
        return "extension-basename-policy"
    return "zpy-heuristic"


def matching_path_rule(rel: str, policy: Policy) -> dict[str, Any] | None:
    for rule in policy.path_rules:
        if not mode_matches_policy_rule(rule, policy.mode, policy.package_role):
            continue
        pattern = str(rule.get("pattern", ""))
        if pattern and (fnmatch.fnmatch(rel, pattern) or fnmatch.fnmatch(Path(rel).name, pattern)):
            return rule
    return None


def is_codex_control_rel(rel: str, current_run: str) -> bool:
    rel = rel.strip("/")
    for variant in codex_rel_variants(rel):
        current_script = f"scripts/{normalize_codex_run_id(current_run).lower()}_verify.sh"
        if variant in {"scripts/verify.sh", current_script, CODEX_ARTIFACT_CLASSIFICATION}:
            return False
        if variant in {
            CODEX_RUN_INDEX,
            CODEX_CURRENT_RUN,
            CODEX_ARCHIVAL_POLICY,
            CODEX_ARTIFACT_CLASSIFICATION,
        }:
            return True
        if variant.startswith("docs/codex-runs/"):
            return True
        if variant.startswith(("handoffs/", "prompts/", "tasks/")):
            return True
        if variant.startswith("docs/") and path_has_current_run_marker(variant, current_run):
            return True
        if variant.startswith("scripts/") and path_has_current_run_marker(variant, current_run):
            return True
    return False


def include_decision(path: Path, archive_root: Path, reserved_output_paths: set[Path], policy: Policy) -> tuple[bool, str]:
    try:
        resolved = path.resolve()
    except OSError:
        return False, "unresolvable-path"
    rel = to_posix(safe_relative(path, archive_root))

    try:
        lstat_result = path.lstat()
    except OSError:
        return False, "stat-failed"
    if not stat.S_ISREG(lstat_result.st_mode) and not stat.S_ISLNK(lstat_result.st_mode):
        return False, "special-file"

    if resolved in reserved_output_paths:
        return False, "generated-output"

    if policy.package_role == "release-context" and is_codex_control_rel(rel, policy.codex_current_run):
        return False, "package-role-codex-control-disabled"

    if is_fixture_sidecar_rel(rel):
        return True, "included-fixture-sidecar"

    if is_generated_sidecar_path(path):
        return False, "generated-sidecar"

    if is_codex_archive_rel(rel) and not policy.include_codex_archive:
        return False, "codex-archive-disabled"

    if stale_codex_reason_for_rel(rel, policy.codex_current_run, policy.codex_artifact_classification):
        return False, "stale-codex-artifact-disabled"

    if path.is_symlink() and not policy.follow_symlinks:
        return False, "symlink-disabled"

    if path.is_symlink() and policy.follow_symlinks:
        try:
            target = path.resolve(strict=True)
        except OSError:
            return False, "broken-symlink"
        if not is_relative_to(target, archive_root):
            return False, "symlink-target-outside-root"

    if not policy.allow_secret_like_names and is_secret_like_path(path):
        return False, "secret-like-filename"

    try:
        size = path.stat().st_size
    except OSError:
        return False, "stat-failed"

    if policy.max_file_size_bytes and size > policy.max_file_size_bytes:
        return False, "max-file-size-exceeded"

    suffix = path.suffix.lower()
    rule = matching_path_rule(rel, policy)
    if rule and rule.get("decision") in {"exclude", "quarantine", "archive-root"}:
        return False, f"policy-{rule.get('decision')}: {rule.get('reason', rule.get('pattern', rel))}"
    if suffix in ARCHIVE_EXTENSIONS:
        return False, "archive-file"
    if suffix in BINARY_EXTENSIONS:
        return False, "binary-build-artifact"
    if suffix in DATABASE_EXTENSIONS:
        return False, "database-file"
    if suffix in DOC_BINARY_EXTENSIONS and not policy.include_doc_binaries:
        return False, "doc-binary-disabled"
    if suffix in IMAGE_EXTENSIONS and not policy.include_images:
        return False, "image-disabled"
    if suffix in LOG_EXTENSIONS:
        if is_context_receipt_log(rel, policy.package_role):
            text_reason = text_file_policy_reason(path, limit_bytes=1024 * 1024)
            if text_reason:
                return False, text_reason
            return True, "included-context-receipt-log"
        if not policy.include_logs:
            return False, "log-disabled"
    if suffix in PATCH_EVIDENCE_EXTENSIONS:
        if policy.include_patch_artifacts:
            text_reason = text_file_policy_reason(path, limit_bytes=1024 * 1024)
            if text_reason:
                return False, text_reason
            return True, "included-patch-evidence"
        return False, "patch-evidence-disabled"

    if policy.profile == "semantic-memory" and rel in {
        "semantic-memory/Cargo.lock",
        "stack-ids/Cargo.lock",
        "semantic-memory-forge/Cargo.lock",
        "forge-memory-bridge/Cargo.lock",
    }:
        return False, "member-lockfile-pruned-for-packaged-workspace"

    if path.name.lower() in ALLOWED_ENV_SAMPLE_NAMES:
        text_reason = text_file_policy_reason(path, limit_bytes=1024 * 1024)
        if text_reason:
            return False, text_reason
        return True, "included-env-sample"
    if allowed_basename(path):
        text_reason = text_file_policy_reason(path, limit_bytes=1024 * 1024)
        if text_reason:
            return False, text_reason
        return True, "included-basename"
    if path.name in policy.allowed_basenames:
        text_reason = text_file_policy_reason(path, limit_bytes=1024 * 1024)
        if text_reason:
            return False, text_reason
        return True, "policy-included-basename"
    if rule and rule.get("decision") == "include":
        text_reason = text_file_policy_reason(path, limit_bytes=1024 * 1024)
        if text_reason:
            return False, text_reason
        return True, f"policy-include: {rule.get('reason', rule.get('pattern', rel))}"
    if suffix in ALLOWED_TEXT_EXTENSIONS:
        text_reason = text_file_policy_reason(path, limit_bytes=1024 * 1024)
        if text_reason:
            return False, text_reason
        return True, "included-extension"
    if suffix in policy.allowed_extensions:
        text_reason = text_file_policy_reason(path, limit_bytes=1024 * 1024)
        if text_reason:
            return False, text_reason
        return True, "policy-included-extension"
    if suffix in DOC_BINARY_EXTENSIONS and policy.include_doc_binaries:
        return True, "included-doc-binary"
    if suffix in IMAGE_EXTENSIONS and policy.include_images:
        return True, "included-image"
    if suffix in LOG_EXTENSIONS and policy.include_logs:
        return True, "included-log"
    return False, "unsupported-extension-or-basename"


def collect_files(
    archive_root: Path,
    include_roots: Sequence[Path],
    reserved_output_paths: set[Path],
    policy: Policy,
) -> tuple[list[Path], list[ExcludedEntry], list[PrunedDirEntry], list[Finding], list[DecisionEntry]]:
    included: list[Path] = []
    excluded: list[ExcludedEntry] = []
    pruned: list[PrunedDirEntry] = []
    findings: list[Finding] = []
    decisions: list[DecisionEntry] = []
    seen_files: set[Path] = set()

    def consider_file(path: Path) -> None:
        try:
            resolved = path.resolve()
        except OSError:
            resolved = path.absolute()
        if resolved in seen_files:
            return
        seen_files.add(resolved)

        try:
            rel = safe_relative(path, archive_root)
        except ValueError:
            rel = path
        include, reason = include_decision(path, archive_root, reserved_output_paths, policy)
        reserved_component = windows_reserved_path(to_posix(rel))
        if reserved_component:
            findings.append(Finding(
                code="windows-reserved-path",
                severity="error" if policy.fail_on_windows_reserved_name else "warning",
                path=to_posix(rel),
                detail=f"Path component {reserved_component!r} is reserved on Windows.",
            ))
        if include:
            included.append(path)
            decisions.append(DecisionEntry(
                path=to_posix(rel),
                decision="include",
                reason=reason,
                source=decision_source_for_reason(reason),
                mode=policy.mode,
            ))
        else:
            excluded.append(ExcludedEntry(path=to_posix(rel), reason=reason))
            decisions.append(DecisionEntry(
                path=to_posix(rel),
                decision="exclude",
                reason=reason,
                source=decision_source_for_reason(reason),
                mode=policy.mode,
            ))
            if reason in {"secret-like-filename", "symlink-target-outside-root", "broken-symlink", "special-file"}:
                findings.append(Finding(
                    code=reason,
                    severity="error" if reason != "secret-like-filename" else "warning",
                    path=to_posix(rel),
                    detail=f"File excluded because of {reason}.",
                ))

    for include_root in include_roots:
        for dirpath, dirnames, filenames in os.walk(include_root, topdown=True, followlinks=policy.follow_symlinks):
            current = Path(dirpath)
            keep_dirs: list[str] = []
            for dirname in sorted(dirnames):
                rel_dir = safe_relative(current / dirname, archive_root)
                reason = should_prune_dir(rel_dir, dirname, policy)
                if reason:
                    pruned.append(PrunedDirEntry(path=to_posix(rel_dir), reason=reason))
                    decisions.append(DecisionEntry(
                        path=to_posix(rel_dir),
                        decision="prune-dir",
                        reason=reason,
                        source=decision_source_for_reason(reason),
                        mode=policy.mode,
                    ))
                else:
                    keep_dirs.append(dirname)
            dirnames[:] = keep_dirs

            for filename in sorted(filenames):
                consider_file(current / filename)

    if policy.profile != "semantic-memory" and not is_under_any(archive_root, include_roots):
        for path in sorted(archive_root.iterdir(), key=lambda p: p.name):
            if path.is_file():
                consider_file(path)

    included.sort(key=lambda p: to_posix(safe_relative(p, archive_root)))
    excluded.sort(key=lambda e: e.path)
    pruned.sort(key=lambda e: e.path)
    decisions.sort(key=lambda d: (d.path, d.decision, d.reason))
    return included, excluded, pruned, findings, decisions


def path_exists_any(root: Path, alternatives: Sequence[str]) -> bool:
    return any((root / alt).exists() for alt in alternatives)


def has_cargo_member(root: Path) -> bool:
    for path in root.rglob("Cargo.toml"):
        if path == root / "Cargo.toml":
            continue
        if any(part in ALWAYS_EXCLUDED_DIR_NAMES for part in path.parts):
            continue
        return True
    return False


def collect_cargo_dependency_path_refs(parsed: dict[str, Any]) -> list[str]:
    refs: list[str] = []

    def collect_dependency_table(table: Any) -> None:
        if not isinstance(table, dict):
            return
        for dep_spec in table.values():
            if isinstance(dep_spec, dict):
                path_value = dep_spec.get("path")
                if isinstance(path_value, str):
                    refs.append(path_value)

    for table_name in CARGO_DEP_TABLE_NAMES:
        collect_dependency_table(parsed.get(table_name))

    workspace = parsed.get("workspace")
    if isinstance(workspace, dict):
        for table_name in CARGO_DEP_TABLE_NAMES:
            collect_dependency_table(workspace.get(table_name))

    target = parsed.get("target")
    if isinstance(target, dict):
        for target_spec in target.values():
            if not isinstance(target_spec, dict):
                continue
            for table_name in CARGO_DEP_TABLE_NAMES:
                collect_dependency_table(target_spec.get(table_name))

    patch = parsed.get("patch")
    if isinstance(patch, dict):
        for registry_patch in patch.values():
            collect_dependency_table(registry_patch)

    replace = parsed.get("replace")
    if isinstance(replace, dict):
        collect_dependency_table(replace)

    return refs


def cargo_path_refs(cargo_toml: Path) -> list[str]:
    text = read_text_lossy(cargo_toml)
    if text is None:
        return []

    refs: list[str] = []
    if tomllib is not None:
        try:
            parsed = tomllib.loads(text)
        except tomllib.TOMLDecodeError:
            parsed = None
        if parsed is not None:
            refs.extend(collect_cargo_dependency_path_refs(parsed))
    if not refs:
        refs.extend(match.group(1) for match in CARGO_PATH_DEP_RE.finditer(text))

    seen: set[str] = set()
    deduped: list[str] = []
    for ref in refs:
        if ref not in seen:
            seen.add(ref)
            deduped.append(ref)
    return deduped


def iter_cargo_manifests_under(root: Path, policy: Policy) -> list[Path]:
    manifests: list[Path] = []
    for dirpath, dirnames, filenames in os.walk(root, topdown=True, followlinks=policy.follow_symlinks):
        current = Path(dirpath)
        keep_dirs: list[str] = []
        for dirname in sorted(dirnames):
            rel_dir = safe_relative(current / dirname, root)
            if should_prune_dir(rel_dir, dirname, policy) is None:
                keep_dirs.append(dirname)
        dirnames[:] = keep_dirs
        if "Cargo.toml" in filenames:
            manifests.append(current / "Cargo.toml")
    manifests.sort(key=lambda p: to_posix(safe_relative(p, root)))
    return manifests


def cargo_package_root(path_ref: Path) -> Path | None:
    if path_ref.is_dir() and (path_ref / "Cargo.toml").exists():
        return path_ref
    if path_ref.is_file() and path_ref.name == "Cargo.toml":
        return path_ref.parent
    return None


def is_under_any(path: Path, roots: Sequence[Path]) -> bool:
    return any(is_relative_to(path, root) for root in roots)


def rel_parts(path: Path, root: Path) -> tuple[str, ...]:
    return tuple(to_posix(safe_relative(path, root)).split("/"))


def has_any_path_part(path: Path, root: Path, names: set[str]) -> bool:
    wanted = {name.lower() for name in names}
    return any(part.lower() in wanted for part in rel_parts(path, root))


def is_third_party_source_path(path: Path, root: Path) -> bool:
    return has_any_path_part(path, root, THIRD_PARTY_SOURCE_DIR_NAMES)


def is_advisory_rust_ref_path(path: Path, root: Path) -> bool:
    return is_third_party_source_path(path, root) or has_any_path_part(path, root, ADVISORY_RUST_REF_DIR_NAMES)


def is_project_script_path(path: Path, root: Path) -> bool:
    return has_any_path_part(path, root, PROJECT_SCRIPT_DIR_NAMES)


def advisory_context_severity(policy: Policy, default: str = "error") -> str:
    if policy.package_role in {"next-codex-context", "research-context", "codex-run-full", "audit-full"}:
        return "warning"
    if policy.profile in {"research", "generic"}:
        return "warning"
    return default


def dedupe_roots(roots: Sequence[Path]) -> list[Path]:
    ordered: list[Path] = []
    for root in sorted({path.resolve() for path in roots}, key=lambda p: (len(p.parts), to_posix(p))):
        if not is_under_any(root, ordered):
            ordered.append(root)
    return sorted(ordered, key=to_posix)


def common_archive_root(roots: Sequence[Path]) -> Path:
    if not roots:
        raise ValueError("at least one include root is required")
    return Path(os.path.commonpath([str(root.resolve()) for root in roots])).resolve()


def discover_cargo_path_roots(root: Path, policy: Policy) -> list[Path]:
    roots: list[Path] = [root.resolve()]
    scanned_manifests: set[Path] = set()
    index = 0

    while index < len(roots):
        current_root = roots[index]
        index += 1

        for cargo_toml in iter_cargo_manifests_under(current_root, policy):
            resolved_manifest = cargo_toml.resolve()
            if resolved_manifest in scanned_manifests:
                continue
            scanned_manifests.add(resolved_manifest)

            for ref in cargo_path_refs(cargo_toml):
                dep_root = cargo_package_root((cargo_toml.parent / ref).resolve())
                if dep_root is None:
                    continue
                dep_root = dep_root.resolve()
                if not is_under_any(dep_root, roots):
                    roots.append(dep_root)

    return dedupe_roots(roots)


def check_required_surfaces(root: Path, profile: str, mode: str) -> list[Finding]:
    findings: list[Finding] = []
    package_role = package_role_for_mode(mode)

    def require(code: str, alternatives: Sequence[str], detail: str, severity: str = "error") -> None:
        if not path_exists_any(root, alternatives):
            findings.append(Finding(
                code=code,
                severity=severity,
                path="/",
                detail=f"Missing {' or '.join(alternatives)}. {detail}",
            ))

    if profile == "aidens":
        require("missing-cargo-toml", ["Cargo.toml"], "AiDENs handoffs should include the workspace manifest.")
        require("missing-cargo-lock", ["Cargo.lock"], "AiDENs handoffs should pin dependency state.")
        require("missing-source-root", ["crates", "src"], "AiDENs should expose canonical source roots.")
        require("missing-agents", ["AGENTS.md", "agents.md", "AIDENS.md", "aidens.md"], "Codex needs the architectural doctrine file.")
        require("missing-readme", ["README.md", "README", "SOURCE_BASIS.md"], "A human/code-agent entry point is required.")
        if package_role in {"next-codex-context", "codex-run-full", "audit-full"}:
            require("missing-scripts-dir", ["scripts"], "Scripts are expected for validation/assertion gates.", severity="warning")
            require("missing-evals-dir", ["evals", "evaluations"], "Evals are expected for stronger handoff packages.", severity="warning")
            require("missing-fixtures-dir", ["fixtures", "tests/fixtures"], "Fixtures are frequently needed by tests and include references.", severity="warning")
            require("missing-handoff-context", ["prompts", "handoffs", "docs"], "Codex-context mode should include guidance/context surfaces.", severity="warning")

    elif profile == "libraries":
        require("missing-cargo-toml", ["Cargo.toml"], "Libraries workspace should include the root manifest.")
        require("missing-cargo-lock", ["Cargo.lock"], "Libraries workspace should include lockfile for reproducible review.")
        if not has_cargo_member(root):
            findings.append(Finding(
                code="missing-cargo-members",
                severity="warning",
                path="/",
                detail="No nested Cargo.toml files were found. If this is a workspace, the archive may be incomplete.",
            ))
        require("missing-readme-or-source-basis", ["README.md", "README", "SOURCE_BASIS.md"], "Libraries handoffs need at least one source-basis/entry document.", severity="warning")

    elif profile in {"recall", "recall-coding"}:
        require("missing-cargo-toml", ["Cargo.toml"], "Recall-family projects should include the Rust workspace manifest.")
        require("missing-cargo-lock", ["Cargo.lock"], "Recall-family packages should include dependency lock state.", severity="warning")
        require("missing-source-root", ["src", "crates", "recall-app", "recall-daemon", "recall-session", "ui", "src-tauri"], "Expected Recall source/UI/daemon surface not found.")
        require("missing-readme", ["README.md", "README", "SOURCE_BASIS.md"], "A source-basis or README is expected.", severity="warning")
        if package_role in {"next-codex-context", "codex-run-full", "audit-full"}:
            require("missing-agents", ["AGENTS.md", "agents.md"], "Codex-context mode should include an agent instruction file.", severity="warning")

    elif profile == "semantic-memory":
        require("missing-cargo-toml", ["Cargo.toml"], "semantic-memory profile expects the crate manifest.")
        require("missing-source-root", ["src"], "semantic-memory profile expects the crate src/ tree.")
        require("missing-audit-gates", ["01_ACCEPTANCE_GATES.sh"], "semantic-memory stabilization handoffs should include acceptance gates.", severity="warning")

    elif profile == "generic-rust":
        require("missing-cargo-toml", ["Cargo.toml"], "generic-rust profile expects a Rust manifest.")
        require("missing-source-root", ["src", "crates"], "generic-rust profile expects src/ or crates/.", severity="warning")

    elif profile == "research":
        if not list(root.glob("*.md")) and not list(root.rglob("*.md")):
            findings.append(Finding(
                code="missing-research-docs",
                severity="warning",
                path="/",
                detail="research profile found no Markdown files. Confirm this is the intended root.",
            ))

    return findings


def check_context_package_evidence(
    root: Path,
    archive_root: Path,
    included: Sequence[Path],
    policy: Policy,
    synthetic_rels: Sequence[str] = (),
) -> list[Finding]:
    findings: list[Finding] = []
    if policy.package_role not in CONTEXT_LOG_ROLES:
        return findings

    included_root_rels = {
        to_posix(safe_relative(path, root))
        for path in included
        if is_relative_to(path, root)
    }
    included_root_rels.update(rel.strip("/") for rel in synthetic_rels if rel.strip("/"))
    for required in ("python/poly_kv/_native.pyi", "python/poly_kv/py.typed"):
        if (root / required).exists() and required not in included_root_rels:
            findings.append(Finding(
                code="context-package-required-file-not-archived",
                severity="error",
                path=required,
                detail="Required Python sidecar file exists in the repo but is absent from the package manifest.",
            ))

    command_evidence = [rel for rel in included_root_rels if is_context_command_evidence_rel(rel)]
    if not command_evidence:
        findings.append(Finding(
            code="context-package-command-evidence-missing",
            severity="error",
            path="/",
            detail=(
                "Context/audit package manifest must include command-run evidence "
                "(commands_run.log, commands_run.receipts.jsonl, COMMAND_RECEIPTS.jsonl, "
                "COMMAND_EXECUTION_RECEIPTS.jsonl, or *_COMMANDS_RUN.md)."
            ),
        ))

    return findings


def check_policy_required_files(root: Path, included: Sequence[Path], policy: Policy) -> list[Finding]:
    findings: list[Finding] = []
    if not policy.required_files:
        return findings
    included_rels = {
        to_posix(safe_relative(path, root))
        for path in included
        if is_relative_to(path, root)
    }
    for required in policy.required_files:
        if not required_file_applies(required, policy.mode, policy.package_role):
            continue
        rel = str(required.get("path", "")).strip("/")
        if not rel:
            continue
        severity = str(required.get("severity", "error"))
        if severity not in {"warning", "error"}:
            severity = "error"
        if rel not in included_rels:
            detail = str(required.get("reason", "Required by PackagePolicyV1 but absent from the package manifest."))
            findings.append(Finding(
                code="package-policy-required-file-missing",
                severity=severity,
                path=rel,
                detail=detail,
            ))
    return findings


def is_safe_archive_name(name: str) -> bool:
    if not name or "\\" in name or "\x00" in name:
        return False
    path = Path(name)
    if path.is_absolute():
        return False
    if re.match(r"^[A-Za-z]:", name):
        return False
    return all(part not in {"", ".", ".."} for part in name.split("/"))


def windows_reserved_path(rel: str) -> str | None:
    for part in rel.split("/"):
        stem = part.split(".")[0].upper().rstrip(" ")
        if stem in WINDOWS_RESERVED_BASENAMES:
            return part
    return None


def check_portability_gates(files: Sequence[FileEntry], policy: Policy) -> list[Finding]:
    findings: list[Finding] = []
    normalized: dict[str, str] = {}
    casefolded: dict[str, str] = {}
    for entry in files:
        rel = entry.path
        if not is_safe_archive_name(rel):
            findings.append(Finding(
                code="archive-entry-unsafe-path",
                severity="error",
                path=rel,
                detail="Archive entry path is absolute, traversing, empty, drive-rooted, or contains an unsafe separator.",
            ))
        normalized_key = unicodedata.normalize("NFC", rel)
        prior = normalized.get(normalized_key)
        if prior and prior != rel:
            findings.append(Finding(
                code="unicode-normalization-collision",
                severity="error" if policy.fail_on_unicode_collision else "warning",
                path=rel,
                detail=f"Path collides with {prior} after NFC normalization.",
            ))
        else:
            normalized[normalized_key] = rel
        case_key = rel.casefold()
        prior_case = casefolded.get(case_key)
        if prior_case and prior_case != rel:
            findings.append(Finding(
                code="case-insensitive-path-collision",
                severity="error" if policy.fail_on_case_collision else "warning",
                path=rel,
                detail=f"Path collides with {prior_case} on case-insensitive filesystems.",
            ))
        else:
            casefolded[case_key] = rel
        reserved = windows_reserved_path(rel)
        if reserved:
            findings.append(Finding(
                code="windows-reserved-path",
                severity="error" if policy.fail_on_windows_reserved_name else "warning",
                path=rel,
                detail=f"Path component {reserved!r} is reserved on Windows.",
            ))
    return findings


def adapter_severity(policy: Policy, ecosystem: str) -> str:
    value = policy.ecosystem_parity_adapters.get(ecosystem, policy.ecosystem_parity_default_severity)
    if value not in {"off", "info", "warning", "error"}:
        return "info"
    return value


def ecosystem_finding(severity: str, code: str, path: str, detail: str) -> dict[str, str]:
    return {"severity": severity, "code": code, "path": path, "detail": detail}


def existing_rels(root: Path, patterns: Sequence[str]) -> list[str]:
    found: list[str] = []
    for pattern in patterns:
        for path in root.rglob(pattern):
            if path.is_file():
                if any(part in ALWAYS_EXCLUDED_DIR_NAMES for part in path.parts):
                    continue
                found.append(to_posix(safe_relative(path, root)))
    return sorted(set(found))


def command_status(command: str) -> tuple[bool, str]:
    exe = command.split()[0]
    if shutil.which(exe):
        return True, "available-not-run"
    return False, "missing-not-run"


def make_adapter_result(
    ecosystem: str,
    detected: bool,
    manifests: list[str],
    command: str | None,
    expected_files: list[str],
    included_rels: set[str],
    findings: list[dict[str, str]],
) -> EcosystemAdapterResult:
    available = False
    status = "not-applicable"
    if command and detected:
        available, status = command_status(command)
        if detected and not available:
            findings.append(ecosystem_finding(
                "info",
                f"{ecosystem}-dry-run-command-missing",
                "/",
                f"Optional ecosystem dry-run command was not executed because `{command.split()[0]}` is unavailable.",
            ))
    missing = sorted(rel for rel in expected_files if rel not in included_rels)
    return EcosystemAdapterResult(
        ecosystem=ecosystem,
        detected=detected,
        manifests=manifests,
        dry_run_available=available,
        dry_run_command=command,
        dry_run_status=status,
        expected_files=sorted(set(expected_files)),
        missing_from_zpy_package=missing,
        extra_in_zpy_package=[],
        findings=findings,
    )


def docker_copy_add_sources(dockerfile: Path) -> list[str]:
    text = read_text_lossy(dockerfile)
    if text is None:
        return []
    sources: list[str] = []
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split()
        if not parts or parts[0].upper() not in {"COPY", "ADD"}:
            continue
        args = [part for part in parts[1:] if not part.startswith("--")]
        if len(args) >= 2:
            for source in args[:-1]:
                if source.startswith(("http://", "https://")):
                    continue
                sources.append(source.strip('"').strip("'").rstrip("/"))
    return sources


def run_ecosystem_adapters(root: Path, included_rels: set[str], policy: Policy) -> tuple[list[EcosystemAdapterResult], list[Finding]]:
    if not policy.ecosystem_parity_enabled:
        return [], []
    results: list[EcosystemAdapterResult] = []
    findings: list[Finding] = []

    def add_result(result: EcosystemAdapterResult) -> None:
        results.append(result)
        severity = adapter_severity(policy, result.ecosystem)
        if severity == "off":
            return
        for rel in result.missing_from_zpy_package:
            findings.append(Finding(
                code=f"{result.ecosystem}-expected-file-not-packaged",
                severity=severity if severity in {"warning", "error"} else "warning",
                path=rel,
                detail=f"{result.ecosystem} adapter expected this existing file to be included.",
            ))
        for item in result.findings:
            item_severity = item["severity"] if item["severity"] != "info" else severity
            if item_severity == "off":
                continue
            findings.append(Finding(
                code=item["code"],
                severity=item_severity,
                path=item["path"],
                detail=item["detail"],
            ))

    rust_manifests = existing_rels(root, ["Cargo.toml"])
    rust_expected = [rel for rel in ("Cargo.toml", "Cargo.lock", "README.md", "LICENSE") if (root / rel).exists()]
    add_result(make_adapter_result("rust", bool(rust_manifests), rust_manifests, "cargo package --list --allow-dirty", rust_expected, included_rels, []))

    python_manifests = existing_rels(root, ["pyproject.toml", "setup.cfg", "setup.py", "MANIFEST.in"])
    python_expected = [rel for rel in ("pyproject.toml", "setup.cfg", "setup.py", "MANIFEST.in") if (root / rel).exists()]
    python_expected.extend(existing_rels(root, ["py.typed", "*.pyi"]))
    add_result(make_adapter_result("python", bool(python_manifests), python_manifests, "python -m build --sdist --wheel", python_expected, included_rels, []))

    node_manifests = [rel for rel in ("package.json", "package-lock.json", "pnpm-lock.yaml", "yarn.lock") if (root / rel).exists()]
    node_expected = [rel for rel in ("package.json", "README.md", "LICENSE") if (root / rel).exists()]
    add_result(make_adapter_result("node", bool(node_manifests), node_manifests, "npm pack --dry-run --json", node_expected, included_rels, []))

    go_manifests = [rel for rel in ("go.mod", "go.sum", "go.work") if (root / rel).exists()]
    add_result(make_adapter_result("go", bool(go_manifests), go_manifests, "go list -m -json all", go_manifests, included_rels, []))

    docker_manifests = [rel for rel in ("Dockerfile", "Containerfile", ".dockerignore") if (root / rel).exists()]
    docker_expected = list(docker_manifests)
    docker_findings: list[dict[str, str]] = []
    for docker_rel in ("Dockerfile", "Containerfile"):
        docker_path = root / docker_rel
        if not docker_path.exists():
            continue
        for source in docker_copy_add_sources(docker_path):
            source_path = root / source
            if source_path.exists() and source_path.is_file():
                docker_expected.append(to_posix(safe_relative(source_path, root)))
            elif not any(ch in source for ch in "*?["):
                docker_findings.append(ecosystem_finding(
                    "warning",
                    "docker-copy-add-source-missing",
                    docker_rel,
                    f"Dockerfile references missing COPY/ADD source: {source}",
                ))
    add_result(make_adapter_result("docker", bool(docker_manifests), docker_manifests, "docker buildx build --progress=plain --dry-run .", docker_expected, included_rels, docker_findings))

    git_manifests = [rel for rel in (".gitignore", ".gitattributes") if (root / rel).exists()]
    git_detected = (root / ".git").exists() or bool(git_manifests)
    git_expected = list(git_manifests)
    git_findings = []
    if (root / ".git").exists():
        git_findings.append(ecosystem_finding("info", "git-metadata-excluded", ".git/", "Git metadata detected and intentionally excluded from transferable package contents."))
    add_result(make_adapter_result("git", git_detected, git_manifests, "git archive --format=tar HEAD", git_expected, included_rels, git_findings))

    return results, findings


def nearest_cargo_manifest_dir(path: Path, root: Path) -> Path | None:
    current = path.parent
    root_resolved = root.resolve()
    while True:
        if (current / "Cargo.toml").exists():
            return current
        if current.resolve() == root_resolved or current.parent == current:
            return None
        current = current.parent


def check_rust_include_refs(root: Path, included: Sequence[Path], policy: Policy) -> list[Finding]:
    findings: list[Finding] = []
    included_resolved = {p.resolve() for p in included}

    for path in included:
        if path.suffix.lower() != ".rs":
            continue
        if is_third_party_source_path(path, root):
            continue
        text = read_text_lossy(path)
        if text is None:
            continue
        rel = to_posix(safe_relative(path, root))
        severity = "warning" if is_advisory_rust_ref_path(path, root) else advisory_context_severity(policy)

        for match in INCLUDE_LITERAL_RE.finditer(text):
            ref = match.group(1)
            if ref.startswith("$") or "{" in ref or "}" in ref:
                continue
            target = (path.parent / ref).resolve()
            if not is_relative_to(target, root):
                findings.append(Finding(
                    code="rust-include-ref-outside-root",
                    severity=severity,
                    path=rel,
                    detail=f"include_str!/include_bytes! reference points outside archive root: {ref}",
                ))
            elif not target.exists():
                findings.append(Finding(
                    code="rust-include-ref-missing",
                    severity=severity,
                    path=rel,
                    detail=f"include_str!/include_bytes! reference does not exist: {ref}",
                ))
            elif target not in included_resolved:
                findings.append(Finding(
                    code="rust-include-ref-not-archived",
                    severity=severity,
                    path=rel,
                    detail=f"include_str!/include_bytes! target exists but is not included in archive: {to_posix(safe_relative(target, root))}",
                ))

        for match in INCLUDE_CARGO_MANIFEST_RE.finditer(text):
            ref = match.group(1).lstrip("/")
            manifest_dir = nearest_cargo_manifest_dir(path, root)
            if manifest_dir is None:
                findings.append(Finding(
                    code="rust-include-cargo-manifest-dir-unresolved",
                    severity="warning",
                    path=rel,
                    detail=f"Could not resolve CARGO_MANIFEST_DIR for include reference: {ref}",
                ))
                continue
            target = (manifest_dir / ref).resolve()
            if not is_relative_to(target, root):
                findings.append(Finding(
                    code="rust-include-ref-outside-root",
                    severity=severity,
                    path=rel,
                    detail=f"CARGO_MANIFEST_DIR include reference points outside archive root: {ref}",
                ))
            elif not target.exists():
                findings.append(Finding(
                    code="rust-include-ref-missing",
                    severity=severity,
                    path=rel,
                    detail=f"CARGO_MANIFEST_DIR include reference does not exist: {ref}",
                ))
            elif target not in included_resolved:
                findings.append(Finding(
                    code="rust-include-ref-not-archived",
                    severity=severity,
                    path=rel,
                    detail=f"CARGO_MANIFEST_DIR include target exists but is not included in archive: {to_posix(safe_relative(target, root))}",
                ))

    return findings


def check_cargo_path_deps(root: Path, included: Sequence[Path], allow_external: bool, policy: Policy) -> list[Finding]:
    findings: list[Finding] = []
    included_resolved = {p.resolve() for p in included}
    cargo_tomls = [p for p in included if p.name == "Cargo.toml"]
    missing_severity = advisory_context_severity(policy)

    for cargo in cargo_tomls:
        if is_third_party_source_path(cargo, root):
            continue
        rel = to_posix(safe_relative(cargo, root))
        for dep in cargo_path_refs(cargo):
            dep_path = (cargo.parent / dep).resolve()
            if not dep_path.exists():
                findings.append(Finding(
                    code="cargo-path-dep-missing",
                    severity=missing_severity,
                    path=rel,
                    detail=f"Cargo path dependency does not exist: {dep}",
                ))
                continue
            if not is_relative_to(dep_path, root):
                findings.append(Finding(
                    code="cargo-path-dep-outside-root",
                    severity="warning" if allow_external else advisory_context_severity(policy),
                    path=rel,
                    detail=f"Cargo path dependency points outside archive root: {dep}",
                ))
                continue
            dep_manifest = dep_path / "Cargo.toml" if dep_path.is_dir() else dep_path
            if dep_manifest.exists() and dep_manifest.resolve() not in included_resolved:
                findings.append(Finding(
                    code="cargo-path-dep-not-archived",
                    severity=advisory_context_severity(policy),
                    path=rel,
                    detail=f"Cargo path dependency exists but its manifest is not included: {to_posix(safe_relative(dep_manifest, root))}",
                ))
    return findings


def check_script_refs(root: Path, included: Sequence[Path], policy: Policy) -> list[Finding]:
    findings: list[Finding] = []
    included_resolved = {p.resolve() for p in included}
    script_suffixes = {".sh", ".bash", ".zsh"}

    def script_project_root(script: Path) -> Path:
        current = script.parent
        root_resolved = root.resolve()
        while True:
            if (current / "z.py").exists() or (current / "Cargo.toml").exists():
                return current
            if current.resolve() == root_resolved or current.parent == current:
                return root
            current = current.parent

    for path in included:
        if path.suffix.lower() not in script_suffixes:
            continue
        if is_third_party_source_path(path, root) or not is_project_script_path(path, root):
            continue
        text = read_text_lossy(path)
        if text is None:
            continue
        rel = to_posix(safe_relative(path, root))
        if is_codex_archive_rel(rel):
            continue
        severity = advisory_context_severity(policy)
        for line in text.splitlines():
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            for regex in SCRIPT_REF_RES:
                for match in regex.finditer(stripped):
                    ref = match.group(1)
                    project_root = script_project_root(path)
                    candidates = [
                        (path.parent / ref).resolve(),
                        (project_root / ref).resolve(),
                        (root / ref).resolve(),
                    ]
                    if any(candidate.exists() for candidate in candidates):
                        for candidate in candidates:
                            if candidate.exists() and is_relative_to(candidate, root) and candidate.resolve() not in included_resolved:
                                findings.append(Finding(
                                    code="script-ref-not-archived",
                                    severity=severity,
                                    path=rel,
                                    detail=f"Script reference exists but is not included: {ref}",
                                ))
                        continue
                    findings.append(Finding(
                        code="script-ref-missing",
                        severity=severity,
                        path=rel,
                        detail=f"Possible script reference not found: {ref}",
                    ))
    return findings


def check_secret_content(root: Path, included: Sequence[Path], policy: Policy) -> list[Finding]:
    findings: list[Finding] = []
    for path in included:
        if is_third_party_source_path(path, root):
            continue
        suffix = path.suffix.lower()
        if suffix not in ALLOWED_TEXT_EXTENSIONS and not allowed_basename(path) and path.name.lower() not in ALLOWED_ENV_SAMPLE_NAMES:
            continue
        try:
            size = path.stat().st_size
        except OSError:
            continue
        if size > policy.secret_scan_max_bytes:
            continue
        text = read_text_lossy(path, limit_bytes=policy.secret_scan_max_bytes)
        if not text:
            continue
        rel = to_posix(safe_relative(path, root))
        for pattern_name, regex, severity in SECRET_CONTENT_PATTERNS:
            match = first_reportable_secret_match(pattern_name, regex, text, path, root)
            if match:
                line_no = text[: match.start()].count("\n") + 1
                findings.append(Finding(
                    code=f"secret-content-{pattern_name}",
                    severity=severity,
                    path=rel,
                    detail=f"Potential secret-like content detected at line {line_no}; value intentionally not printed.",
                ))
    return findings


def is_benign_secret_match(pattern_name: str, path: Path, root: Path, text: str, match: re.Match[str]) -> bool:
    line_start = text.rfind("\n", 0, match.start()) + 1
    line_end = text.find("\n", match.end())
    if line_end == -1:
        line_end = len(text)
    line = text[line_start:line_end].lower()
    rel = to_posix(safe_relative(path, root)).lower()
    parts = set(rel.split("/"))

    placeholderish = any(token in line for token in SECRET_PLACEHOLDER_TOKENS)
    if path.name.lower() in ALLOWED_ENV_SAMPLE_NAMES and placeholderish:
        return True

    if pattern_name in {"openai-like-key", "github-token", "private-key-block"}:
        fixture_context = bool(parts & {"examples", "fixtures", "reference", "test", "testdata", "tests"})
        repeated_digits = "1234567890" in match.group(0) or "abcdef" in match.group(0).lower()
        if pattern_name in {"openai-like-key", "github-token"} and fixture_context:
            return True
        if repeated_digits and pattern_name != "private-key-block":
            return True
        if placeholderish and pattern_name != "private-key-block":
            return True

    return False


def first_reportable_secret_match(
    pattern_name: str,
    regex: re.Pattern[str],
    text: str,
    path: Path,
    root: Path,
) -> re.Match[str] | None:
    for match in regex.finditer(text):
        if pattern_name == "named-secret-assignment" and is_non_literal_rust_secret_forwarding(text, match):
            continue
        if is_benign_secret_match(pattern_name, path, root, text, match):
            continue
        return match
    return None


def is_non_literal_rust_secret_forwarding(text: str, match: re.Match[str]) -> bool:
    line_start = text.rfind("\n", 0, match.start()) + 1
    line_end = text.find("\n", match.end())
    if line_end == -1:
        line_end = len(text)
    line = text[line_start:line_end]
    snippet = match.group(0)
    delimiter_positions = [pos for pos in (snippet.find(":"), snippet.find("=")) if pos != -1]
    if not delimiter_positions:
        return False
    rhs = snippet[min(delimiter_positions) + 1 :].strip()
    tail = line[match.end() - line_start :].lstrip()
    if tail.startswith("()"):
        rhs = f"{rhs}()"
    if not rhs or "'" in rhs or '"' in rhs:
        return False
    return bool(RUST_FIELD_FORWARDING_SECRET_ASSIGNMENT_RE.fullmatch(rhs))


def file_entry_for_path(root: Path, path: Path) -> FileEntry:
    rel = to_posix(safe_relative(path, root))
    return FileEntry(
        path=rel,
        bytes=path.stat().st_size,
        sha256=sha256_file(path),
        mode=mode_string(path),
        executable=is_executable(path),
        mtime_utc=file_mtime_utc(path),
    )


def build_file_entries(root: Path, included: Sequence[Path]) -> tuple[list[FileEntry], list[Finding]]:
    entries: list[FileEntry] = []
    findings: list[Finding] = []
    for path in included:
        rel = to_posix(safe_relative(path, root))
        try:
            entries.append(file_entry_for_path(root, path))
        except OSError as exc:
            findings.append(Finding(
                code="included-file-disappeared",
                severity="error",
                path=rel,
                detail=f"File was selected for packaging but could not be read during manifest creation: {exc}",
            ))
    return entries, findings


def file_entry_for_synthetic(synthetic: SyntheticFile) -> FileEntry:
    return FileEntry(
        path=synthetic.path,
        bytes=len(synthetic.data),
        sha256=hashlib.sha256(synthetic.data).hexdigest(),
        mode=f"{synthetic.mode:06o}",
        executable=bool(synthetic.mode & stat.S_IXUSR),
        mtime_utc="1980-01-01T00:00:00Z",
    )


def toml_workspace_version(workspace_manifest: Path, name: str, fallback: str) -> str:
    if tomllib is None or not workspace_manifest.exists():
        return fallback
    try:
        with workspace_manifest.open("rb") as f:
            data = tomllib.load(f)
        value = data.get("workspace", {}).get("dependencies", {}).get(name)
    except (OSError, tomllib.TOMLDecodeError):
        return fallback
    if isinstance(value, str):
        return value
    if isinstance(value, dict) and isinstance(value.get("version"), str):
        return value["version"]
    return fallback


def table_dep(version: str, features: Sequence[str] | None = None) -> str:
    if not features:
        return f'"{version}"'
    rendered_features = ", ".join(f'"{feature}"' for feature in features)
    return f'{{ version = "{version}", features = [{rendered_features}] }}'


def semantic_memory_workspace_manifest(archive_root: Path) -> bytes:
    parent_manifest = archive_root / "Cargo.toml"

    def version(name: str, fallback: str) -> str:
        return toml_workspace_version(parent_manifest, name, fallback)

    body = f"""# Generated by semantic-memory/z.py for hermetic review archives.
[workspace]
resolver = "2"
members = [
  "semantic-memory",
  "stack-ids",
  "semantic-memory-forge",
  "forge-memory-bridge",
]
default-members = ["semantic-memory"]

[workspace.dependencies]
rusqlite = {table_dep(version("rusqlite", "0.32.1"), ["bundled", "blob"])}
serde = {table_dep(version("serde", "1.0.228"), ["derive"])}
serde_json = {table_dep(version("serde_json", "1.0.149"))}
tokio = {table_dep(version("tokio", "1.50.0"), ["rt", "macros", "sync"])}
thiserror = {table_dep(version("thiserror", "2.0.18"))}
tracing = {table_dep(version("tracing", "0.1.44"))}
uuid = {table_dep(version("uuid", "1.22.0"), ["v4"])}
chrono = {table_dep(version("chrono", "0.4.44"), ["serde"])}
schemars = {table_dep(version("schemars", "0.8.22"))}
tempfile = {table_dep(version("tempfile", "3.27.0"))}
proptest = {table_dep(version("proptest", "1.10.0"))}

[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "allow"

[workspace.lints.clippy]
todo = "deny"
dbg_macro = "deny"
unimplemented = "deny"
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
"""
    return body.encode("utf-8")


def context_command_receipt(
    root: Path,
    archive_root: Path,
    output_path: Path,
    policy: Policy,
) -> bytes:
    record = {
        "schema": "ZpyCommandReceiptV1",
        "authority": "receipt",
        "tool": Path(__file__).name,
        "script_version": SCRIPT_VERSION,
        "created_utc": utc_now_iso(),
        "command_argv": [Path(sys.argv[0]).name, *sys.argv[1:]],
        "root": str(root),
        "archive_root": str(archive_root),
        "output": str(output_path),
        "profile": policy.profile,
        "mode": policy.mode,
        "package_role": policy.package_role,
        "status": "package-command-recorded",
        "note": "Generated by z.py for this package invocation; no repository source file was required.",
    }
    return (json.dumps(record, sort_keys=True) + "\n").encode("utf-8")


def synthetic_files_for_profile(
    archive_root: Path,
    included: Sequence[Path],
    profile: str,
    *,
    root: Path | None = None,
    output_path: Path | None = None,
    policy: Policy | None = None,
) -> list[SyntheticFile]:
    included_rels = {to_posix(safe_relative(path, archive_root)) for path in included}
    synthetic: list[SyntheticFile] = []

    if (
        policy is not None
        and output_path is not None
        and policy.package_role in CONTEXT_LOG_ROLES
        and not any(is_context_command_evidence_rel(rel) for rel in included_rels)
    ):
        synthetic.append(SyntheticFile(
            CONTEXT_COMMAND_RECEIPT_SYNTHETIC_PATH,
            context_command_receipt(root or archive_root, archive_root, output_path, policy),
        ))

    if profile != "semantic-memory":
        return synthetic

    if "Cargo.toml" not in included_rels:
        synthetic.append(SyntheticFile("Cargo.toml", semantic_memory_workspace_manifest(archive_root)))

    root_lock = archive_root / "Cargo.lock"
    if "Cargo.lock" not in included_rels and root_lock.exists():
        synthetic.append(SyntheticFile("Cargo.lock", root_lock.read_bytes()))

    return synthetic


def deterministic_zip_time(source_date_epoch: int | None) -> tuple[int, int, int, int, int, int]:
    if source_date_epoch is not None:
        return datetime.fromtimestamp(source_date_epoch, UTC).timetuple()[:6]
    env_epoch = source_date_epoch_from_value(None)
    if env_epoch is not None:
        return datetime.fromtimestamp(env_epoch, UTC).timetuple()[:6]
    return ZIP_EPOCH


def zip_info_for_file(path: Path, arcname: str, deterministic: bool, source_date_epoch: int | None = None) -> zipfile.ZipInfo:
    info = zipfile.ZipInfo(arcname)
    if deterministic:
        info.date_time = deterministic_zip_time(source_date_epoch)
    else:
        info.date_time = datetime.fromtimestamp(path.stat().st_mtime).timetuple()[:6]
    mode = stat.S_IMODE(path.stat().st_mode)
    info.external_attr = ((stat.S_IFREG | mode) & 0xFFFF) << 16
    return info


def zip_info_for_synthetic(synthetic: SyntheticFile, deterministic: bool, source_date_epoch: int | None = None) -> zipfile.ZipInfo:
    info = zipfile.ZipInfo(synthetic.path)
    info.date_time = deterministic_zip_time(source_date_epoch) if deterministic else datetime.now().timetuple()[:6]
    info.external_attr = ((stat.S_IFREG | synthetic.mode) & 0xFFFF) << 16
    return info


def write_archive(
    root: Path,
    output_path: Path,
    included: Sequence[Path],
    deterministic: bool,
    compresslevel: int,
    synthetic_files: Sequence[SyntheticFile] = (),
    source_date_epoch: int | None = None,
) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output_path, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=compresslevel) as zf:
        for synthetic in synthetic_files:
            info = zip_info_for_synthetic(synthetic, deterministic=deterministic, source_date_epoch=source_date_epoch)
            zf.writestr(info, synthetic.data, compress_type=zipfile.ZIP_DEFLATED, compresslevel=compresslevel)
        for path in included:
            arcname = to_posix(safe_relative(path, root))
            info = zip_info_for_file(path, arcname, deterministic=deterministic, source_date_epoch=source_date_epoch)
            with path.open("rb") as f:
                zf.writestr(info, f.read(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=compresslevel)


def write_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def severity_counts(findings: Sequence[Finding]) -> tuple[int, int]:
    errors = sum(1 for f in findings if f.severity == "error")
    warnings = sum(1 for f in findings if f.severity == "warning")
    return errors, warnings


def summarize_extensions(files: Sequence[FileEntry]) -> dict[str, int]:
    counter: Counter[str] = Counter()
    for entry in files:
        suffix = Path(entry.path).suffix.lower() or "<no-extension>"
        counter[suffix] += 1
    return dict(sorted(counter.items(), key=lambda item: (-item[1], item[0])))


def summarize_top_dirs(files: Sequence[FileEntry]) -> dict[str, int]:
    counter: Counter[str] = Counter()
    for entry in files:
        parts = Path(entry.path).parts
        top = parts[0] if parts else "."
        counter[top] += 1
    return dict(sorted(counter.items(), key=lambda item: (-item[1], item[0])))


def summarize_exclusion_reasons(excluded: Sequence[ExcludedEntry]) -> dict[str, int]:
    counter = Counter(e.reason for e in excluded)
    return dict(sorted(counter.items(), key=lambda item: (-item[1], item[0])))


def render_markdown_report(result: BuildResult, extension_summary: dict[str, int], top_dir_summary: dict[str, int], exclusion_summary: dict[str, int]) -> str:
    report = result.report
    lines: list[str] = []
    lines.append(f"# Zip Source Certifier Report")
    lines.append("")
    lines.append("## Summary")
    lines.append("")
    lines.append(f"- Script version: `{report.script_version}`")
    lines.append(f"- Created UTC: `{report.created_utc}`")
    lines.append(f"- Root: `{report.root}`")
    lines.append(f"- Archive root: `{report.archive_root}`")
    lines.append(f"- Output: `{report.output}`")
    lines.append(f"- Include roots: `{len(report.include_roots)}`")
    lines.append(f"- External Cargo path dependency roots: `{len(report.external_path_dep_roots)}`")
    lines.append(f"- Profile: `{report.profile_resolved}` requested as `{report.profile_requested}`")
    lines.append(f"- Mode: `{report.mode}`")
    lines.append(f"- Package role: `{report.package_role}`")
    lines.append(f"- Strict: `{report.strict}`")
    lines.append(f"- Dry run: `{report.dry_run}`")
    if report.policy_path:
        lines.append(f"- Package policy: `{report.policy_path}`")
    lines.append(f"- Included files: `{report.included_count}`")
    lines.append(f"- Included bytes: `{report.included_bytes}`")
    lines.append(f"- Excluded files: `{report.excluded_file_count}`")
    lines.append(f"- Pruned dirs: `{report.pruned_dir_count}`")
    lines.append(f"- Findings: `{report.findings_count}` (`{report.error_count}` errors, `{report.warning_count}` warnings)")
    if report.archive_sha256:
        lines.append(f"- Archive zip-byte SHA-256: `{report.archive_zip_byte_sha256}`")
        lines.append(f"- Archive hash semantics: `{report.archive_sha256_semantics}`")
    if report.content_manifest_sha256:
        lines.append(f"- Content manifest SHA-256: `{report.content_manifest_sha256}`")
    if report.ecosystem_parity:
        detected = [item["ecosystem"] for item in report.ecosystem_parity if item.get("detected")]
        lines.append(f"- Ecosystems detected: `{', '.join(detected) if detected else 'none'}`")
    if report.codex_archive:
        codex = report.codex_archive
        lines.append(f"- Codex archive enabled: `{codex.get('enabled')}`")
        lines.append(f"- Codex archive planned: `{codex.get('planned_count')}`")
        lines.append(f"- Codex archive moved: `{codex.get('moved_count')}`")
        lines.append(f"- Codex active stale after normalization: `{codex.get('active_stale_after_count')}`")
    if report.root_markdown_archive:
        root_md = report.root_markdown_archive
        lines.append(f"- Root Markdown archive enabled: `{root_md.get('enabled')}`")
        lines.append(f"- Root Markdown inspected: `{root_md.get('inspected_count')}`")
        lines.append(f"- Root Markdown protected: `{root_md.get('protected_count')}`")
        lines.append(f"- Root Markdown candidates: `{root_md.get('candidate_count')}`")
        lines.append(f"- Root Markdown ambiguous: `{root_md.get('ambiguous_count')}`")
        lines.append(f"- Root Markdown moved: `{root_md.get('moved_count')}`")
        lines.append(f"- Root Markdown collisions: `{root_md.get('collision_count')}`")
    if report.root_package_archive:
        root_pkg = report.root_package_archive
        lines.append(f"- Root package archive enabled: `{root_pkg.get('enabled')}`")
        lines.append(f"- Root package inspected: `{root_pkg.get('inspected_count')}`")
        lines.append(f"- Root package protected: `{root_pkg.get('protected_count')}`")
        lines.append(f"- Root package candidates: `{root_pkg.get('candidate_count')}`")
        lines.append(f"- Root package moved: `{root_pkg.get('moved_count')}`")
        lines.append(f"- Root package skipped existing: `{root_pkg.get('skipped_existing_count')}`")
        lines.append(f"- Root package collisions: `{root_pkg.get('collision_count')}`")
    lines.append("")

    lines.append("## Ecosystem parity")
    lines.append("")
    if report.ecosystem_parity:
        lines.append("| Ecosystem | Detected | Manifests | Missing expected | Dry-run status |")
        lines.append("|---|---:|---:|---:|---|")
        for adapter in report.ecosystem_parity:
            lines.append(
                f"| `{adapter.get('ecosystem')}` | `{adapter.get('detected')}` | "
                f"{len(adapter.get('manifests', []))} | {len(adapter.get('missing_from_zpy_package', []))} | "
                f"`{adapter.get('dry_run_status')}` |"
            )
    else:
        lines.append("No ecosystem adapters were run.")
    lines.append("")

    lines.append("## Decision provenance")
    lines.append("")
    lines.append(f"- Decisions recorded: `{len(result.decisions)}`")
    if result.decisions:
        include_count = sum(1 for decision in result.decisions if decision.decision == "include")
        exclude_count = sum(1 for decision in result.decisions if decision.decision == "exclude")
        prune_count = sum(1 for decision in result.decisions if decision.decision == "prune-dir")
        lines.append(f"- Includes: `{include_count}`")
        lines.append(f"- Excludes: `{exclude_count}`")
        lines.append(f"- Pruned dirs: `{prune_count}`")
    lines.append("")

    lines.append("## Validation findings")
    lines.append("")
    if not result.findings:
        lines.append("No validation findings.")
    else:
        lines.append("| Severity | Code | Path | Detail |")
        lines.append("|---|---|---|---|")
        for finding in result.findings:
            detail = finding.detail.replace("|", "\\|")
            path = finding.path.replace("|", "\\|")
            lines.append(f"| {finding.severity} | `{finding.code}` | `{path}` | {detail} |")
    lines.append("")

    lines.append("## Included files by extension")
    lines.append("")
    if extension_summary:
        lines.append("| Extension | Count |")
        lines.append("|---|---:|")
        for ext, count in extension_summary.items():
            lines.append(f"| `{ext}` | {count} |")
    else:
        lines.append("No included files.")
    lines.append("")

    lines.append("## Included files by top-level path")
    lines.append("")
    if top_dir_summary:
        lines.append("| Top-level path | Count |")
        lines.append("|---|---:|")
        for top, count in top_dir_summary.items():
            lines.append(f"| `{top}` | {count} |")
    else:
        lines.append("No included files.")
    lines.append("")

    lines.append("## Exclusion reasons")
    lines.append("")
    if exclusion_summary:
        lines.append("| Reason | Count |")
        lines.append("|---|---:|")
        for reason, count in exclusion_summary.items():
            lines.append(f"| `{reason}` | {count} |")
    else:
        lines.append("No excluded files were recorded.")
    lines.append("")

    lines.append("## Sidecar files")
    lines.append("")
    for label, value in [
        ("Manifest", report.manifest_path),
        ("Markdown report", report.report_path),
        ("Excluded file list", report.excluded_path),
        ("Findings", report.findings_path),
    ]:
        if value:
            lines.append(f"- {label}: `{value}`")
    lines.append("")

    lines.append("## Interpretation")
    lines.append("")
    if report.error_count:
        lines.append("This package has validation errors. Under `--strict`, it should not be treated as a complete handoff until corrected or explicitly waived.")
    elif report.warning_count:
        lines.append("This package has warnings. It is probably usable, but the warnings should be reviewed before using it as a Codex or audit handoff.")
    else:
        lines.append("This package passed the configured validation gates.")
    lines.append("")
    return "\n".join(lines)


def default_output_path(root: Path, resolved_profile: str, mode: str) -> Path:
    stamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    safe_profile = resolved_profile.replace("/", "-")
    safe_mode = mode.replace("/", "-")
    return root / f"{root.name}-{safe_profile}-{safe_mode}-{stamp}.zip"


def output_sidecar_path(output_path: Path, suffix: str, explicit: str | None) -> Path | None:
    if explicit == "-":
        return None
    if explicit:
        return Path(explicit).expanduser().resolve()
    return output_path.with_suffix(suffix)


def validate_root(root: Path) -> None:
    if not root.exists():
        raise FileNotFoundError(f"root does not exist: {root}")
    if not root.is_dir():
        raise NotADirectoryError(f"root is not a directory: {root}")


def resolve_codex_archive_root(root: Path, value: str) -> Path:
    archive_root = Path(value).expanduser()
    if not archive_root.is_absolute():
        archive_root = root / archive_root
    return archive_root.resolve()


def resolve_root_markdown_archive_root(root: Path, value: str) -> Path:
    archive_root = Path(value).expanduser()
    if not archive_root.is_absolute():
        archive_root = root / archive_root
    return archive_root.resolve()


def resolve_root_package_archive_root(root: Path, value: str) -> Path:
    archive_root = Path(value).expanduser()
    if not archive_root.is_absolute():
        archive_root = root / archive_root
    return archive_root.resolve()


def root_markdown_archive_stamp() -> str:
    return datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")


def root_package_archive_stamp() -> str:
    return datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")


def default_codex_archive_report_path(output_path: Path, explicit: str | None) -> Path | None:
    if explicit == "-":
        return None
    if explicit:
        return Path(explicit).expanduser().resolve()
    return output_path.with_suffix(".codex-archive.json")


def iter_codex_archive_candidates(root: Path, current_run: str, archive_root: Path) -> list[CodexArchiveCandidate]:
    candidates: list[CodexArchiveCandidate] = []
    classification = load_codex_artifact_classification(root)
    root_markdown_archive_root = resolve_root_markdown_archive_root(root, ROOT_MARKDOWN_ARCHIVE_DIR)
    ignored_dirs = {".git", "target", "__pycache__"}
    for dirpath, dirnames, filenames in os.walk(root, topdown=True):
        current = Path(dirpath)
        keep_dirs: list[str] = []
        for dirname in sorted(dirnames):
            path = current / dirname
            rel = to_posix(safe_relative(path, root))
            if (
                dirname in ignored_dirs
                or is_codex_archive_dir_rel(rel)
                or is_relative_to(path, archive_root)
                or is_relative_to(path, root_markdown_archive_root)
            ):
                continue
            keep_dirs.append(dirname)
        dirnames[:] = keep_dirs

        for filename in sorted(filenames):
            path = current / filename
            rel = to_posix(safe_relative(path, root))
            if rel in classification:
                continue
            reason = stale_codex_reason_for_rel(rel, current_run, classification)
            if reason is None:
                continue
            candidates.append(CodexArchiveCandidate(
                original_path=rel,
                run_id=infer_codex_run_id(rel, reason),
                reason=reason,
                sha256=sha256_file(path),
                bytes=path.stat().st_size,
                mtime_utc=file_mtime_utc(path),
            ))
    return sorted(candidates, key=lambda c: (c.run_id, c.original_path))


def root_markdown_candidate_matches(filename: str) -> list[str]:
    upper = Path(filename).name.upper()
    return [pattern for pattern in ROOT_MARKDOWN_CANDIDATE_PATTERNS if fnmatch.fnmatch(upper, pattern)]


def classify_root_markdown_candidate(filename: str, current_run: str) -> tuple[str, str]:
    upper = filename.upper()
    if upper in ROOT_MARKDOWN_PROTECTED_FILES_UPPER:
        matches = root_markdown_candidate_matches(filename)
        if matches:
            return "ambiguous", "ambiguous-stop: protected-root-doc"
        return "protected", ""

    if path_has_current_run_marker(filename, current_run):
        matches = root_markdown_candidate_matches(filename)
        if matches:
            return "ambiguous", "ambiguous-stop: active-current-run"
        return "active-current-run", ""

    matches = root_markdown_candidate_matches(filename)
    if len(matches) > 1:
        return "ambiguous", f"ambiguous-stop: {','.join(sorted(matches))}"
    if len(matches) == 1:
        return "candidate", matches[0]
    return "ambiguous", "ambiguous-stop: unknown-root-markdown"


def iter_root_markdown_archive_candidates(root: Path, current_run: str) -> tuple[
    list[tuple[str, str, str]],
    list[str],
    list[str],
    list[str],
]:
    inspected: list[str] = []
    candidates: list[tuple[str, str, str]] = []
    protected: list[str] = []
    ambiguous: list[tuple[str, str]] = []

    for path in sorted(root.iterdir()):
        if not path.is_file() or path.suffix.lower() != ".md":
            continue
        filename = path.name
        inspected.append(filename)
        if classify_root_package_artifact(filename):
            protected.append(filename)
            continue
        category, reason = classify_root_markdown_candidate(filename, current_run)
        if category == "candidate":
            candidates.append((filename, reason or "root-markdown-noise", "candidate-archive"))
        elif category == "protected":
            protected.append(filename)
        elif category == "ambiguous":
            ambiguous.append((filename, reason or "ambiguous-root-markdown"))
        elif category == "active-current-run":
            ambiguous.append((filename, reason or "ambiguous-stop: active-current-run"))

    return (
        candidates,
        inspected,
        protected,
        [f"{filename}:{reason}" for filename, reason in ambiguous],
    )


def make_root_markdown_archive_record(root: Path, filename: str, archived_path: Path, sha256: str, bytes_: int, mtime_utc: str, reason: str, classification: str) -> dict[str, Any]:
    return {
        "original_path": to_posix((root / filename).name),
        "archived_path": to_posix(safe_relative(archived_path, root)),
        "sha256": sha256,
        "bytes": bytes_,
        "mtime_utc": mtime_utc,
        "reason": reason,
        "classification": classification,
    }


def is_root_package_protected_file(name: str) -> bool:
    if name in ROOT_PACKAGE_PROTECTED_FILE_PATTERNS:
        return True
    upper = name.upper()
    return any(
        upper == prefix
        or upper.startswith(prefix + ".")
        or upper.startswith(prefix + "_")
        or upper.startswith(prefix + "-")
        for prefix in ROOT_PACKAGE_PROTECTED_PREFIXES
    )


def classify_root_package_artifact(filename: str) -> str | None:
    if is_root_package_protected_file(filename):
        return None
    for pattern, reason in ROOT_PACKAGE_ARTIFACT_PATTERNS:
        if pattern.match(filename):
            return reason
    return None


def iter_root_package_archive_candidates(
    root: Path,
    reserved_output_paths: set[Path],
    archive_root: Path,
) -> tuple[list[tuple[str, str]], list[str], list[str]]:
    inspected: list[str] = []
    protected: list[str] = []
    candidates: list[tuple[str, str]] = []

    for path in sorted(root.iterdir(), key=lambda p: p.name):
        if not path.is_file():
            continue
        if is_relative_to(path, archive_root):
            continue
        inspected.append(path.name)
        try:
            resolved = path.resolve()
        except OSError:
            protected.append(path.name)
            continue
        if resolved in reserved_output_paths:
            protected.append(path.name)
            continue
        if is_root_package_protected_file(path.name):
            protected.append(path.name)
            continue
        reason = classify_root_package_artifact(path.name)
        if reason:
            candidates.append((path.name, reason))

    return candidates, inspected, protected


def make_root_package_archive_record(root: Path, filename: str, archived_path: Path, sha256: str, bytes_: int, mtime_utc: str, reason: str) -> dict[str, Any]:
    return {
        "original_path": filename,
        "archived_path": to_posix(safe_relative(archived_path, root)),
        "sha256": sha256,
        "bytes": bytes_,
        "mtime_utc": mtime_utc,
        "reason": reason,
    }


def archive_root_package_artifacts(
    root: Path,
    args: argparse.Namespace,
    reserved_output_paths: set[Path],
    *,
    enabled: bool,
    dry_run: bool,
    verify_only: bool,
) -> RootPackageArchiveResult:
    archive_root = resolve_root_package_archive_root(root, args.root_package_archive_root)
    archive_dir = archive_root / root_package_archive_stamp()
    manifest_path = archive_dir / ROOT_PACKAGE_ARCHIVE_MANIFEST

    candidates, inspected, protected = iter_root_package_archive_candidates(root, reserved_output_paths, archive_root)
    planned: list[dict[str, Any]] = []
    moved: list[dict[str, Any]] = []
    skipped_existing: list[dict[str, Any]] = []
    collisions: list[dict[str, Any]] = []
    errors: list[str] = []
    operations: list[dict[str, Any]] = []

    for filename, reason in candidates:
        source = root / filename
        try:
            source_sha256 = sha256_file(source)
            source_bytes = source.stat().st_size
            mtime_utc = file_mtime_utc(source)
        except OSError as exc:
            errors.append(f"failed to inspect root package artifact {filename}: {exc}")
            continue
        requested_dest = archive_dir / "files" / filename
        dest, collision, same_existing = unique_archive_destination(requested_dest, source_sha256)
        record = make_root_package_archive_record(
            root,
            filename,
            dest,
            source_sha256,
            source_bytes,
            mtime_utc,
            reason,
        )
        planned.append(record)
        if collision:
            collision = dict(collision)
            collision["original_path"] = filename
            collisions.append(collision)
            errors.append(
                f"failed to archive {filename}: destination collision for existing file with different content."
            )
            continue
        operations.append({
            "filename": filename,
            "source": source,
            "dest": dest,
            "same_existing": same_existing,
            "record": record,
        })

    manifest_written = False
    should_move = enabled and not dry_run and not verify_only and not errors
    if should_move:
        for operation in operations:
            source = operation["source"]
            dest = operation["dest"]
            record = operation["record"]
            same_existing = operation["same_existing"]
            if same_existing:
                skipped_existing.append(record)
                try:
                    source.unlink()
                except OSError as exc:
                    errors.append(f"failed to remove active duplicate after archived copy was found: {operation['filename']}: {exc}")
                continue
            try:
                dest.parent.mkdir(parents=True, exist_ok=True)
                source.rename(dest)
                moved.append(record)
            except OSError as exc:
                errors.append(f"failed to archive {operation['filename']}: {exc}")

    if enabled and should_move and not errors:
        manifest_payload = {
            "root_package_archive_manifest_version": ROOT_PACKAGE_ARCHIVE_MANIFEST_VERSION,
            "created_utc": utc_now_iso(),
            "tool": Path(__file__).name,
            "tool_version": SCRIPT_VERSION,
            "repo_root": str(root),
            "archive_root": str(archive_dir),
            "files": moved + skipped_existing,
            "planned": planned,
            "collisions": collisions,
            "errors": errors,
            "summary": {
                "inspected_count": len(inspected),
                "protected_count": len(protected),
                "candidate_count": len(candidates),
                "planned_count": len(planned),
                "moved_count": len(moved),
                "skipped_existing_count": len(skipped_existing),
                "collision_count": len(collisions),
            },
        }
        write_json(manifest_path, manifest_payload)
        manifest_written = True

    return RootPackageArchiveResult(
        enabled=enabled,
        dry_run=dry_run,
        verify_only=verify_only,
        archive_only=bool(args.archive_only),
        archive_root=str(archive_root),
        archive_dir=str(archive_dir),
        manifest_path=str(manifest_path) if manifest_written else None,
        inspected_count=len(inspected),
        protected_count=len(protected),
        candidate_count=len(candidates),
        planned_count=len(planned),
        moved_count=len(moved),
        skipped_existing_count=len(skipped_existing),
        collision_count=len(collisions),
        manifest_written=manifest_written,
        candidate_paths=[filename for filename, _reason in candidates],
        protected_paths=protected,
        moved=moved,
        skipped_existing=skipped_existing,
        collisions=collisions,
        errors=errors,
    )


def archive_root_markdown_noise(
    root: Path,
    args: argparse.Namespace,
    output_path: Path,
    current_run: str,
    *,
    dry_run: bool,
    verify_only: bool,
) -> RootMarkdownArchiveResult:
    archive_root = resolve_root_markdown_archive_root(root, args.root_markdown_archive_root)
    archive_dir = archive_root / root_markdown_archive_stamp()
    manifest_path = archive_dir / ROOT_MARKDOWN_ARCHIVE_MANIFEST

    candidates, inspected, protected, ambiguous = iter_root_markdown_archive_candidates(root, current_run)
    planned: list[dict[str, Any]] = []
    moved: list[dict[str, Any]] = []
    skipped_existing: list[dict[str, Any]] = []
    collisions: list[dict[str, Any]] = []
    errors: list[str] = []
    candidate_paths: list[str] = []
    moved_count = 0
    skipped_existing_count = 0

    operations: list[dict[str, Any]] = []
    for filename, reason, classification in candidates:
        source = root / filename
        candidate_paths.append(filename)
        requested_dest = archive_dir / "files" / filename
        dest, collision, same_existing = unique_archive_destination(requested_dest, sha256_file(source))
        record = make_root_markdown_archive_record(
            root,
            filename,
            dest,
            sha256_file(source),
            source.stat().st_size,
            file_mtime_utc(source),
            reason,
            classification,
        )
        planned.append(record)
        if collision:
            collisions.append({
                "original_path": filename,
                "requested_path": to_posix(safe_relative(requested_dest, root)),
                "resolved_path": to_posix(safe_relative(dest, root)),
                "reason": collision["reason"],
            })
            errors.append(
                f"failed to archive {filename}: destination collision for existing file with different content."
            )
            continue
        operations.append({
            "filename": filename,
            "source": source,
            "dest": dest,
            "same_existing": same_existing,
            "record": record,
        })

    manifest_written = False
    should_move = (
        not dry_run
        and not verify_only
        and not errors
    )
    if should_move:
        for operation in operations:
            source = operation["source"]
            dest = operation["dest"]
            record = operation["record"]
            same_existing = operation["same_existing"]
            if same_existing:
                skipped_existing.append(record)
                skipped_existing_count += 1
                try:
                    source.unlink()
                    prune_empty_parents(source, root)
                except OSError as exc:
                    errors.append(f"failed to remove active duplicate after archived copy was found: {operation['filename']}: {exc}")
                continue
            try:
                dest.parent.mkdir(parents=True, exist_ok=True)
                source.rename(dest)
                moved.append(record)
                moved_count += 1
                prune_empty_parents(source, root)
            except OSError as exc:
                errors.append(f"failed to archive {operation['filename']}: {exc}")
                continue

    if should_move and not errors and manifest_path is not None:
        manifest_payload = {
            "root_markdown_archive_manifest_version": ROOT_MARKDOWN_ARCHIVE_MANIFEST_VERSION,
            "created_utc": utc_now_iso(),
            "tool": Path(__file__).name,
            "tool_version": SCRIPT_VERSION,
            "repo_root": str(root),
            "archive_root": str(archive_dir),
            "current_run": current_run,
            "files": moved + skipped_existing,
            "collisions": collisions,
            "errors": errors,
            "summary": {
                "inspected_count": len(inspected),
                "protected_count": len(protected),
                "candidate_count": len(candidates),
                "ambiguous_count": len(ambiguous),
            },
        }
        write_json(manifest_path, manifest_payload)
        manifest_written = True

    return RootMarkdownArchiveResult(
        enabled=bool(args.archive_root_markdown_noise),
        dry_run=dry_run,
        verify_only=verify_only,
        archive_only=bool(args.archive_only),
        current_run=current_run,
        archive_root=str(archive_root),
        archive_dir=str(archive_dir),
        manifest_path=str(manifest_path),
        inspected_count=len(inspected),
        protected_count=len(protected),
        candidate_count=len(candidates),
        ambiguous_count=len(ambiguous),
        planned_count=len(planned),
        moved_count=moved_count,
        skipped_existing_count=skipped_existing_count,
        collision_count=len(collisions),
        manifest_written=manifest_written,
        candidate_paths=candidate_paths,
        protected_paths=protected,
        ambiguous_paths=ambiguous,
        collisions=collisions,
        errors=errors,
    )


def archive_dir_for_run(archive_root: Path, run_id: str, stamp: str) -> Path:
    if run_id == "unclassified":
        base = archive_root / "unclassified" / stamp
    else:
        base = archive_root / safe_archive_component(run_id)
    if not base.exists():
        return base
    manifest = base / "ARCHIVE_MANIFEST.json"
    if not manifest.exists() and not any(base.iterdir()):
        return base
    if run_id == "unclassified":
        candidate = base
    else:
        candidate = archive_root / f"{safe_archive_component(run_id)}-{stamp}"
    counter = 2
    while candidate.exists() and any(candidate.iterdir()):
        candidate = archive_root / f"{safe_archive_component(run_id)}-{stamp}-{counter}"
        counter += 1
    return candidate


def make_archive_record(root: Path, candidate: CodexArchiveCandidate, archived_path: Path) -> dict[str, Any]:
    return {
        "original_path": candidate.original_path,
        "archived_path": to_posix(safe_relative(archived_path, root)),
        "sha256": candidate.sha256,
        "bytes": candidate.bytes,
        "mtime_utc": candidate.mtime_utc,
        "reason": candidate.reason,
        "run_id": candidate.run_id,
    }


def unique_archive_destination(dest: Path, source_sha256: str) -> tuple[Path, dict[str, Any] | None, bool]:
    if not dest.exists():
        return dest, None, False
    if dest.is_file() and sha256_file(dest) == source_sha256:
        return dest, None, True

    suffix = dest.suffix
    stem = dest.name[: -len(suffix)] if suffix else dest.name
    candidate = dest.with_name(f"{stem}.{source_sha256[:12]}{suffix}")
    counter = 2
    while candidate.exists():
        if candidate.is_file() and sha256_file(candidate) == source_sha256:
            return candidate, None, True
        candidate = dest.with_name(f"{stem}.{source_sha256[:12]}.{counter}{suffix}")
        counter += 1
    return candidate, {
        "requested_path": to_posix(dest),
        "resolved_path": to_posix(candidate),
        "reason": "archive-path-collision",
    }, False


def prune_empty_parents(path: Path, stop: Path) -> None:
    current = path.parent
    stop = stop.resolve()
    while current.resolve() != stop and is_relative_to(current, stop):
        try:
            current.rmdir()
        except OSError:
            break
        current = current.parent


def render_codex_supersession(run_id: str, current_run: str, moved_count: int) -> str:
    return "\n".join([
        f"# Codex Run Supersession - {run_id}",
        "",
        f"Superseded by: `{current_run}`",
        "",
        "This directory contains historical Codex-run material archived out of active repository space.",
        "These files are evidence, not active instructions.",
        "",
        f"Archived file count: `{moved_count}`",
        "",
    ])


def render_codex_run_summary(run_id: str, records: Sequence[dict[str, Any]], created_utc: str) -> str:
    lines = [
        f"# Codex Run Archive Summary - {run_id}",
        "",
        f"Created UTC: `{created_utc}`",
        f"Archived files: `{len(records)}`",
        "",
        "| Original path | Reason | SHA-256 |",
        "|---|---|---|",
    ]
    for record in records:
        lines.append(f"| `{record['original_path']}` | `{record['reason']}` | `{record['sha256']}` |")
    lines.append("")
    return "\n".join(lines)


def write_codex_run_index(root: Path, archive_root: Path, current_run: str, manifest_paths: Sequence[str], created_utc: str) -> None:
    docs_root = root / "docs" / "codex-runs"
    docs_root.mkdir(parents=True, exist_ok=True)
    (docs_root / "CURRENT_RUN.md").write_text(
        "\n".join([
            "# Current Codex Run",
            "",
            f"Current run: `{current_run}`",
            f"Updated UTC: `{created_utc}`",
            "",
            "Historical run material in `docs/codex-runs/archive/` is evidence, not active instruction.",
            "",
        ]),
        encoding="utf-8",
    )
    (docs_root / "ARCHIVAL_POLICY.md").write_text(
        "\n".join([
            "# Codex Run Archival Policy",
            "",
            "`z.py` archives stale Codex-run prompts, tasks, handoffs, and evidence before normal packaging.",
            "Normal `release-context`, `next-codex-context`, and `codex-run-full` packages exclude `docs/codex-runs/archive/` unless `--include-codex-archive` or `--mode audit-full` is explicit.",
            "Existing archive manifests are not rewritten; new collisions are routed to fresh paths.",
            "",
        ]),
        encoding="utf-8",
    )
    lines = [
        "# Codex Run Index",
        "",
        f"Updated UTC: `{created_utc}`",
        f"Archive root: `{to_posix(safe_relative(archive_root, root))}`",
        "",
    ]
    if manifest_paths:
        lines.extend(["## Archive Manifests", ""])
        for path in manifest_paths:
            lines.append(f"- `{path}`")
        lines.append("")
    else:
        lines.append("No run archive manifests were written by this invocation.")
        lines.append("")
    (docs_root / "CODEX_RUN_INDEX.md").write_text("\n".join(lines), encoding="utf-8")


def existing_codex_manifest_paths(root: Path, archive_root: Path) -> list[str]:
    if not archive_root.exists():
        return []
    paths = [
        to_posix(safe_relative(path, root))
        for path in archive_root.rglob("ARCHIVE_MANIFEST.json")
        if path.is_file()
    ]
    return sorted(set(paths))


def archive_codex_run_artifacts(
    root: Path,
    args: argparse.Namespace,
    output_path: Path,
    *,
    dry_run: bool,
    verify_only: bool = False,
) -> CodexArchiveResult:
    current_run = normalize_codex_run_id(args.codex_current_run)
    archive_root = resolve_codex_archive_root(root, args.codex_archive_root)
    report_path = default_codex_archive_report_path(output_path, args.codex_archive_report_out)
    stamp = codex_run_stamp()
    candidates = iter_codex_archive_candidates(root, current_run, archive_root)
    grouped: dict[str, list[CodexArchiveCandidate]] = {}
    for candidate in candidates:
        grouped.setdefault(candidate.run_id, []).append(candidate)

    archive_dirs = {
        run_id: archive_dir_for_run(archive_root, run_id, stamp)
        for run_id in grouped
    }
    planned: list[dict[str, Any]] = []
    moved: list[dict[str, Any]] = []
    skipped_existing: list[dict[str, Any]] = []
    collisions: list[dict[str, Any]] = []
    unclassified: list[dict[str, Any]] = []
    manifest_paths: list[str] = []
    errors: list[str] = []
    records_by_run: dict[str, list[dict[str, Any]]] = {run_id: [] for run_id in grouped}

    for run_id, run_candidates in grouped.items():
        archive_dir = archive_dirs[run_id]
        for candidate in run_candidates:
            source = root / candidate.original_path
            requested_dest = archive_dir / "files" / candidate.original_path
            dest, collision, same_existing = unique_archive_destination(requested_dest, candidate.sha256)
            if collision:
                collision = dict(collision)
                collision["original_path"] = candidate.original_path
                collisions.append(collision)
            record = make_archive_record(root, candidate, dest)
            planned.append(record)
            if run_id == "unclassified":
                unclassified.append(record)
            if same_existing:
                skipped_existing.append(record)
                if not dry_run and not verify_only:
                    try:
                        source.unlink()
                        prune_empty_parents(source, root)
                    except OSError as exc:
                        errors.append(f"failed to remove active duplicate after archived copy was found: {candidate.original_path}: {exc}")
                continue
            records_by_run[run_id].append(record)
            if dry_run or verify_only:
                continue
            try:
                dest.parent.mkdir(parents=True, exist_ok=True)
                source.rename(dest)
                moved.append(record)
                prune_empty_parents(source, root)
            except OSError as exc:
                errors.append(f"failed to archive {candidate.original_path}: {exc}")

    created_utc = utc_now_iso()
    if not dry_run and not verify_only:
        for run_id, records in records_by_run.items():
            archive_dir = archive_dirs[run_id]
            manifest_path = archive_dir / "ARCHIVE_MANIFEST.json"
            if manifest_path.exists():
                errors.append(f"refusing to rewrite existing archive manifest: {to_posix(safe_relative(manifest_path, root))}")
                continue
            archive_dir.mkdir(parents=True, exist_ok=True)
            manifest_payload = {
                "archive_manifest_version": CODEX_ARCHIVE_MANIFEST_VERSION,
                "created_utc": created_utc,
                "tool": Path(__file__).name,
                "tool_version": SCRIPT_VERSION,
                "repo_root": str(root),
                "run_id": run_id,
                "superseded_by": current_run,
                "files": records,
                "collisions": [c for c in collisions if c.get("original_path") in {r["original_path"] for r in records}],
                "skipped_existing": [r for r in skipped_existing if r["run_id"] == run_id],
                "unclassified": [r for r in unclassified if r["run_id"] == run_id],
            }
            write_json(manifest_path, manifest_payload)
            (archive_dir / "SUPERSESSION.md").write_text(
                render_codex_supersession(run_id, current_run, len(records)),
                encoding="utf-8",
            )
            (archive_dir / "RUN_SUMMARY.md").write_text(
                render_codex_run_summary(run_id, records, created_utc),
                encoding="utf-8",
            )
            manifest_paths.append(to_posix(safe_relative(manifest_path, root)))
        docs_root = root / "docs" / "codex-runs"
        if grouped or not (docs_root / "CODEX_RUN_INDEX.md").exists():
            indexed_manifest_paths = existing_codex_manifest_paths(root, archive_root)
            write_codex_run_index(root, archive_root, current_run, indexed_manifest_paths, created_utc)
    else:
        for run_id, archive_dir in archive_dirs.items():
            manifest_paths.append(to_posix(safe_relative(archive_dir / "ARCHIVE_MANIFEST.json", root)))

    if verify_only or not args.archive_codex_runs:
        active_after = [candidate.original_path for candidate in candidates]
    elif dry_run:
        active_after = []
    else:
        active_after = [
            candidate.original_path
            for candidate in iter_codex_archive_candidates(root, current_run, archive_root)
        ]

    result = CodexArchiveResult(
        enabled=bool(args.archive_codex_runs),
        dry_run=dry_run,
        verify_only=verify_only,
        archive_only=bool(args.archive_only),
        current_run=current_run,
        archive_root=str(archive_root),
        report_path=str(report_path) if report_path else None,
        stale_active_before=[candidate.original_path for candidate in candidates],
        planned=planned,
        moved=moved,
        skipped_existing=skipped_existing,
        collisions=collisions,
        unclassified=unclassified,
        active_stale_after=active_after,
        manifest_paths=manifest_paths,
        errors=errors,
    )
    if report_path:
        write_json(report_path, asdict(result))
    return result


def codex_archive_summary(result: CodexArchiveResult | None) -> dict[str, Any] | None:
    if result is None:
        return None
    return {
        "enabled": result.enabled,
        "dry_run": result.dry_run,
        "verify_only": result.verify_only,
        "archive_only": result.archive_only,
        "current_run": result.current_run,
        "archive_root": result.archive_root,
        "report_path": result.report_path,
        "stale_active_before_count": len(result.stale_active_before),
        "planned_count": len(result.planned),
        "moved_count": len(result.moved),
        "skipped_existing_count": len(result.skipped_existing),
        "collision_count": len(result.collisions),
        "unclassified_count": len(result.unclassified),
        "active_stale_after_count": len(result.active_stale_after),
        "manifest_paths": result.manifest_paths,
        "errors": result.errors,
    }


def root_markdown_archive_summary(result: RootMarkdownArchiveResult | None) -> dict[str, Any] | None:
    if result is None:
        return None
    return {
        "enabled": result.enabled,
        "dry_run": result.dry_run,
        "verify_only": result.verify_only,
        "archive_only": result.archive_only,
        "current_run": result.current_run,
        "archive_root": result.archive_root,
        "archive_dir": result.archive_dir,
        "manifest_path": result.manifest_path if result.manifest_written else None,
        "inspected_count": result.inspected_count,
        "protected_count": result.protected_count,
        "candidate_count": result.candidate_count,
        "ambiguous_count": result.ambiguous_count,
        "planned_count": result.planned_count,
        "moved_count": result.moved_count,
        "skipped_existing_count": result.skipped_existing_count,
        "collision_count": result.collision_count,
        "candidate_paths": result.candidate_paths,
        "ambiguous_paths": result.ambiguous_paths,
        "errors": result.errors,
    }


def root_package_archive_summary(result: RootPackageArchiveResult | None) -> dict[str, Any] | None:
    if result is None:
        return None
    return {
        "enabled": result.enabled,
        "dry_run": result.dry_run,
        "verify_only": result.verify_only,
        "archive_only": result.archive_only,
        "archive_root": result.archive_root,
        "archive_dir": result.archive_dir,
        "manifest_path": result.manifest_path,
        "inspected_count": result.inspected_count,
        "protected_count": result.protected_count,
        "candidate_count": result.candidate_count,
        "planned_count": result.planned_count,
        "moved_count": result.moved_count,
        "skipped_existing_count": result.skipped_existing_count,
        "collision_count": result.collision_count,
        "candidate_paths": result.candidate_paths,
        "moved": result.moved,
        "errors": result.errors,
    }


def build_archive_action_result(
    args: argparse.Namespace,
    root: Path,
    resolved_profile: str,
    output_path: Path,
    codex_result: CodexArchiveResult | None,
    root_markdown_result: RootMarkdownArchiveResult | None,
    root_package_result: RootPackageArchiveResult | None,
    findings: Sequence[Finding],
) -> BuildResult:
    error_count, warning_count = severity_counts(findings)
    report_obj = ArchiveReport(
        script=Path(__file__).name,
        script_version=SCRIPT_VERSION,
        created_utc=utc_now_iso(),
        root=str(root),
        archive_root=str(root),
        include_roots=[str(root)],
        external_path_dep_roots=[],
        output=str(output_path),
        profile_requested=args.profile,
        profile_resolved=resolved_profile,
        mode=args.mode,
        package_role=package_role_for_mode(args.mode),
        strict=args.strict,
        dry_run=args.dry_run,
        deterministic_zip_timestamps=not args.preserve_mtime,
        included_count=0,
        included_bytes=0,
        excluded_file_count=0,
        pruned_dir_count=0,
        findings_count=len(findings),
        error_count=error_count,
        warning_count=warning_count,
        archive_sha256=None,
        archive_zip_byte_sha256=None,
        archive_sha256_semantics="zip-byte-sha256-not-canonical-content-hash",
        content_manifest_sha256=None,
        archive_written=False,
        manifest_path=None,
        report_path=None,
        excluded_path=None,
        findings_path=None,
        decision_log_path=None,
        policy_path=getattr(args, "policy", None),
        ecosystem_parity=[],
        codex_archive=codex_archive_summary(codex_result),
        root_markdown_archive=root_markdown_archive_summary(root_markdown_result),
        root_package_archive=root_package_archive_summary(root_package_result),
    )
    return BuildResult(report=report_obj, files=[], excluded=[], pruned_dirs=[], findings=list(findings), decisions=[], ecosystem_parity=[])


def build(args: argparse.Namespace) -> BuildResult:
    root = Path(args.root).expanduser().resolve()
    validate_root(root)

    resolved_profile = infer_profile(root) if args.profile == "auto" else args.profile
    policy = make_policy(args, root, resolved_profile)
    _policy_path, _policy_doc, policy_findings = load_policy_file(getattr(args, "policy", None))

    output_path = Path(args.output).expanduser() if args.output else default_output_path(root, resolved_profile, args.mode)
    if not output_path.is_absolute():
        output_path = (root / output_path).resolve()
    else:
        output_path = output_path.resolve()

    manifest_path = output_sidecar_path(output_path, ".manifest.json", args.manifest_out)
    report_path = output_sidecar_path(output_path, ".report.md", args.report_out)
    excluded_path = output_sidecar_path(output_path, ".excluded.json", args.excluded_out)
    findings_path = output_sidecar_path(output_path, ".findings.json", args.findings_out)
    decision_log_path = output_sidecar_path(output_path, ".decision-log.jsonl", args.decision_log_out) if policy.emit_decision_log or args.decision_log_out else None
    codex_archive_report_path = default_codex_archive_report_path(output_path, args.codex_archive_report_out)
    reserved_output_paths = {
        path.resolve()
        for path in [output_path, manifest_path, report_path, excluded_path, findings_path, codex_archive_report_path]
        if path is not None
    }
    root_package_archive_enabled = (
        args.archive_root_package_artifacts
        if args.archive_root_package_artifacts is not None
        else policy.package_role in ROOT_PACKAGE_ARCHIVE_ROLES
    )

    codex_archive_result: CodexArchiveResult | None
    root_markdown_archive_result: RootMarkdownArchiveResult | None
    root_package_archive_result: RootPackageArchiveResult | None
    archive_findings: list[Finding] = []

    codex_archive_result = None
    root_markdown_archive_result = None
    root_package_archive_result = None
    if args.verify_codex_archive_hygiene:
        codex_archive_result = archive_codex_run_artifacts(
            root,
            args,
            output_path,
            dry_run=True,
            verify_only=True,
        )
        for rel in codex_archive_result.active_stale_after[:50]:
            archive_findings.append(Finding(
                code="codex-archive-hygiene-active-stale",
                severity="error",
                path=rel,
                detail="Stale Codex-run artifact remains active outside docs/codex-runs/archive.",
            ))
        if len(codex_archive_result.active_stale_after) > 50:
            archive_findings.append(Finding(
                code="codex-archive-hygiene-active-stale-truncated",
                severity="error",
                path="/",
                detail=f"{len(codex_archive_result.active_stale_after) - 50} additional stale Codex-run artifacts omitted from console findings.",
            ))

    if args.verify_root_markdown_noise_hygiene:
        root_markdown_archive_result = archive_root_markdown_noise(
            root,
            args,
            output_path,
            current_run=policy.codex_current_run,
            dry_run=True,
            verify_only=True,
        )
        for rel in root_markdown_archive_result.candidate_paths[:50]:
            archive_findings.append(Finding(
                code="root-markdown-hygiene-candidate-remnant",
                severity="error",
                path=rel,
                detail="Root Markdown noise candidate remains in workspace root.",
            ))
        if len(root_markdown_archive_result.candidate_paths) > 50:
            archive_findings.append(Finding(
                code="root-markdown-hygiene-candidate-remnant-truncated",
                severity="error",
                path="/",
                detail=f"{len(root_markdown_archive_result.candidate_paths) - 50} additional root Markdown candidate remnants omitted from console findings.",
            ))
        for path in root_markdown_archive_result.ambiguous_paths:
            archive_findings.append(Finding(
                code="root-markdown-archive-ambiguous",
                severity="error",
                path=path,
                detail="Root Markdown candidate classification is ambiguous.",
            ))
        for collision in root_markdown_archive_result.collisions:
            archive_findings.append(Finding(
                code="root-markdown-archive-collision",
                severity="error",
                path=collision.get("original_path", "/"),
                detail="Root Markdown destination collision prevented movement.",
            ))

    if args.verify_root_package_hygiene:
        root_package_archive_result = archive_root_package_artifacts(
            root,
            args,
            reserved_output_paths,
            enabled=bool(root_package_archive_enabled),
            dry_run=True,
            verify_only=True,
        )
        for rel in root_package_archive_result.candidate_paths[:50]:
            archive_findings.append(Finding(
                code="root-package-hygiene-candidate-remnant",
                severity="error",
                path=rel,
                detail="Root package artifact remains in workspace root.",
            ))
        if len(root_package_archive_result.candidate_paths) > 50:
            archive_findings.append(Finding(
                code="root-package-hygiene-candidate-remnant-truncated",
                severity="error",
                path="/",
                detail=f"{len(root_package_archive_result.candidate_paths) - 50} additional root package artifacts omitted from console findings.",
            ))
        for error in root_package_archive_result.errors:
            archive_findings.append(Finding(
                code="root-package-archive-error",
                severity="error",
                path="/",
                detail=error,
            ))

    if args.verify_codex_archive_hygiene or args.verify_root_markdown_noise_hygiene or args.verify_root_package_hygiene:
        if root_markdown_archive_result is None and args.verify_codex_archive_hygiene:
            root_markdown_candidates, inspected, protected, ambiguous = iter_root_markdown_archive_candidates(
                root,
                policy.codex_current_run,
            )
            root_markdown_archive_root = resolve_root_markdown_archive_root(root, args.root_markdown_archive_root)
            root_markdown_archive_result = RootMarkdownArchiveResult(
                enabled=False,
                dry_run=True,
                verify_only=True,
                archive_only=False,
                current_run=policy.codex_current_run,
                archive_root=str(root_markdown_archive_root),
                archive_dir=str(root_markdown_archive_root),
                manifest_path=None,
                inspected_count=len(inspected),
                protected_count=len(protected),
                candidate_count=len(root_markdown_candidates),
                ambiguous_count=len(ambiguous),
                planned_count=len(root_markdown_candidates),
                moved_count=0,
                skipped_existing_count=0,
                collision_count=0,
                manifest_written=False,
                candidate_paths=[candidate[0] for candidate in root_markdown_candidates],
                protected_paths=protected,
                ambiguous_paths=ambiguous,
                collisions=[],
                errors=[],
            )
        return build_archive_action_result(
            args,
            root,
            resolved_profile,
            output_path,
            codex_archive_result,
            root_markdown_archive_result,
            root_package_archive_result,
            archive_findings,
        )

    if args.archive_codex_runs:
        codex_archive_result = archive_codex_run_artifacts(
            root,
            args,
            output_path,
            dry_run=args.dry_run,
            verify_only=False,
        )
        for error in codex_archive_result.errors:
            archive_findings.append(Finding(
                code="codex-archive-error",
                severity="error",
                path="/",
                detail=error,
            ))
        for rel in codex_archive_result.active_stale_after[:50]:
            archive_findings.append(Finding(
                code="codex-archive-active-stale-after-normalization",
                severity="error",
                path=rel,
                detail="Stale Codex-run artifact remains active after archival normalization.",
            ))
        if len(codex_archive_result.active_stale_after) > 50:
            archive_findings.append(Finding(
                code="codex-archive-active-stale-after-normalization-truncated",
                severity="error",
                path="/",
                detail=f"{len(codex_archive_result.active_stale_after) - 50} additional stale Codex-run artifacts omitted from console findings.",
            ))
    else:
        archive_root_for_scan = resolve_codex_archive_root(root, args.codex_archive_root)
        active_stale = iter_codex_archive_candidates(root, policy.codex_current_run, archive_root_for_scan)
        codex_archive_result = CodexArchiveResult(
            enabled=False,
            dry_run=args.dry_run,
            verify_only=False,
            archive_only=bool(args.archive_only),
            current_run=policy.codex_current_run,
            archive_root=str(archive_root_for_scan),
            report_path=None,
            stale_active_before=[candidate.original_path for candidate in active_stale],
            planned=[],
            moved=[],
            skipped_existing=[],
            collisions=[],
            unclassified=[],
            active_stale_after=[candidate.original_path for candidate in active_stale],
            manifest_paths=[],
            errors=[],
        )
        if args.strict and active_stale:
            archive_findings.append(Finding(
                code="codex-archive-disabled-with-active-stale",
                severity="error",
                path="/",
                detail="--no-archive-codex-runs is diagnostic only; strict packaging cannot proceed with active stale Codex-run artifacts.",
            ))

    root_markdown_candidates, inspected_root_markdown, protected_root_markdown, ambiguous_root_markdown = iter_root_markdown_archive_candidates(
        root,
        policy.codex_current_run,
    )
    if args.archive_root_markdown_noise:
        root_markdown_archive_result = archive_root_markdown_noise(
            root,
            args,
            output_path,
            current_run=policy.codex_current_run,
            dry_run=args.dry_run or args.root_markdown_archive_dry_run,
            verify_only=False,
        )
        for error in root_markdown_archive_result.errors:
            archive_findings.append(Finding(
                code="root-markdown-archive-error",
                severity="error",
                path="/",
                detail=error,
            ))
        if root_markdown_archive_result.dry_run:
            for rel in root_markdown_archive_result.candidate_paths[:50]:
                archive_findings.append(Finding(
                    code="root-markdown-archive-candidate-remains",
                    severity="error" if args.strict else "warning",
                    path=rel,
                    detail="Root Markdown noise candidate remains because archive root pass used dry-run mode.",
                ))
            if len(root_markdown_archive_result.candidate_paths) > 50:
                archive_findings.append(Finding(
                    code="root-markdown-archive-candidate-remains-truncated",
                    severity="error" if args.strict else "warning",
                    path="/",
                    detail=f"{len(root_markdown_archive_result.candidate_paths) - 50} additional root Markdown candidate remnants omitted from console findings.",
                ))
        for path in root_markdown_archive_result.ambiguous_paths:
            archive_findings.append(Finding(
                code="root-markdown-archive-ambiguous",
                severity="error",
                path=path,
                detail="Root Markdown candidate classification is ambiguous.",
            ))
        for collision in root_markdown_archive_result.collisions:
            archive_findings.append(Finding(
                code="root-markdown-archive-collision",
                severity="error",
                path=collision.get("original_path", "/"),
                detail="Root Markdown destination collision prevented movement.",
            ))
    else:
        root_markdown_archive_root = resolve_root_markdown_archive_root(root, args.root_markdown_archive_root)
        root_markdown_archive_result = RootMarkdownArchiveResult(
            enabled=False,
            dry_run=args.dry_run,
            verify_only=False,
            archive_only=bool(args.archive_only),
            current_run=policy.codex_current_run,
            archive_root=str(root_markdown_archive_root),
            archive_dir=str(root_markdown_archive_root),
            manifest_path=None,
            inspected_count=len(inspected_root_markdown),
            protected_count=len(protected_root_markdown),
            candidate_count=len(root_markdown_candidates),
            ambiguous_count=len(ambiguous_root_markdown),
            planned_count=len(root_markdown_candidates),
            moved_count=0,
            skipped_existing_count=0,
            collision_count=0,
            manifest_written=False,
            candidate_paths=[candidate[0] for candidate in root_markdown_candidates],
            protected_paths=protected_root_markdown,
            ambiguous_paths=ambiguous_root_markdown,
            collisions=[],
            errors=[],
        )

    root_package_archive_root = resolve_root_package_archive_root(root, args.root_package_archive_root)
    if root_package_archive_enabled:
        root_package_archive_result = archive_root_package_artifacts(
            root,
            args,
            reserved_output_paths,
            enabled=True,
            dry_run=args.dry_run or args.root_package_archive_dry_run,
            verify_only=False,
        )
        for error in root_package_archive_result.errors:
            archive_findings.append(Finding(
                code="root-package-archive-error",
                severity="error",
                path="/",
                detail=error,
            ))
        if root_package_archive_result.dry_run:
            for rel in root_package_archive_result.candidate_paths[:50]:
                archive_findings.append(Finding(
                    code="root-package-archive-candidate-remains",
                    severity="error" if args.strict else "warning",
                    path=rel,
                    detail="Root package artifact remains because archive root pass used dry-run mode.",
                ))
            if len(root_package_archive_result.candidate_paths) > 50:
                archive_findings.append(Finding(
                    code="root-package-archive-candidate-remains-truncated",
                    severity="error" if args.strict else "warning",
                    path="/",
                    detail=f"{len(root_package_archive_result.candidate_paths) - 50} additional root package artifacts omitted from console findings.",
                ))
        active_after, _inspected_after, _protected_after = iter_root_package_archive_candidates(
            root,
            reserved_output_paths,
            root_package_archive_root,
        )
        for rel, _reason in active_after[:50]:
            archive_findings.append(Finding(
                code="root-package-active-after-normalization",
                severity="error",
                path=rel,
                detail="Root package artifact remains active after archival normalization.",
            ))
        if len(active_after) > 50:
            archive_findings.append(Finding(
                code="root-package-active-after-normalization-truncated",
                severity="error",
                path="/",
                detail=f"{len(active_after) - 50} additional root package artifacts omitted from console findings.",
            ))
    else:
        active_root_package, inspected_root_package, protected_root_package = iter_root_package_archive_candidates(
            root,
            reserved_output_paths,
            root_package_archive_root,
        )
        root_package_archive_result = RootPackageArchiveResult(
            enabled=False,
            dry_run=args.dry_run,
            verify_only=False,
            archive_only=bool(args.archive_only),
            archive_root=str(root_package_archive_root),
            archive_dir=str(root_package_archive_root),
            manifest_path=None,
            inspected_count=len(inspected_root_package),
            protected_count=len(protected_root_package),
            candidate_count=len(active_root_package),
            planned_count=len(active_root_package),
            moved_count=0,
            skipped_existing_count=0,
            collision_count=0,
            manifest_written=False,
            candidate_paths=[candidate[0] for candidate in active_root_package],
            protected_paths=protected_root_package,
            moved=[],
            skipped_existing=[],
            collisions=[],
            errors=[],
        )
        if args.strict and active_root_package:
            archive_findings.append(Finding(
                code="root-package-archive-disabled-with-active-artifacts",
                severity="error",
                path="/",
                detail="Strict packaging cannot proceed while root package artifacts remain active.",
            ))

    if args.archive_only:
        return build_archive_action_result(
            args,
            root,
            resolved_profile,
            output_path,
            codex_archive_result,
            root_markdown_archive_result,
            root_package_archive_result,
            archive_findings,
        )

    include_roots = [root]
    if policy.include_external_path_deps:
        include_roots = discover_cargo_path_roots(root, policy)
    include_roots = dedupe_roots(include_roots)
    archive_root = common_archive_root(include_roots)
    external_path_dep_roots = [path for path in include_roots if path.resolve() != root.resolve()]

    included, excluded, pruned_dirs, collection_findings, decisions = collect_files(
        archive_root,
        include_roots,
        reserved_output_paths,
        policy,
    )
    findings: list[Finding] = []
    findings.extend(policy_findings)
    findings.extend(archive_findings)
    findings.extend(collection_findings)
    findings.extend(check_required_surfaces(root, resolved_profile, args.mode))
    findings.extend(check_policy_required_files(root, included, policy))

    if args.check_rust_include_refs:
        findings.extend(check_rust_include_refs(archive_root, included, policy))
    if args.check_cargo_path_deps:
        findings.extend(check_cargo_path_deps(archive_root, included, allow_external=args.allow_external_path_deps, policy=policy))
    if args.check_script_refs:
        findings.extend(check_script_refs(archive_root, included, policy))
    if args.check_secrets:
        findings.extend(check_secret_content(archive_root, included, policy))

    synthetic_files = synthetic_files_for_profile(
        archive_root,
        included,
        resolved_profile,
        root=root,
        output_path=output_path,
        policy=policy,
    )
    synthetic_paths = {synthetic.path for synthetic in synthetic_files}
    findings.extend(check_context_package_evidence(root, archive_root, included, policy, sorted(synthetic_paths)))
    conflicting_paths = [
        to_posix(safe_relative(path, archive_root))
        for path in included
        if to_posix(safe_relative(path, archive_root)) in synthetic_paths
    ]
    for rel in conflicting_paths:
        findings.append(Finding(
            code="synthetic-file-conflict",
            severity="error",
            path=rel,
            detail="Generated archive root file conflicts with an included file.",
        ))

    # De-duplicate findings while preserving deterministic order.
    seen_finding_keys: set[tuple[str, str, str, str]] = set()
    deduped_findings: list[Finding] = []
    for finding in sorted(findings, key=lambda f: (f.severity, f.code, f.path, f.detail)):
        key = (finding.code, finding.severity, finding.path, finding.detail)
        if key not in seen_finding_keys:
            seen_finding_keys.add(key)
            deduped_findings.append(finding)
    findings = deduped_findings

    file_entries = [file_entry_for_synthetic(synthetic) for synthetic in synthetic_files]
    disk_file_entries, file_entry_findings = build_file_entries(archive_root, included)
    file_entries.extend(disk_file_entries)
    findings.extend(file_entry_findings)
    findings.extend(check_portability_gates(file_entries, policy))
    included_root_rels = {
        entry.path
        for entry in file_entries
        if not entry.path.startswith("../")
    }
    ecosystem_results, ecosystem_findings = run_ecosystem_adapters(root, included_root_rels, policy)
    findings.extend(ecosystem_findings)
    file_entry_payload = [asdict(entry) for entry in file_entries]
    content_manifest_sha256 = sha256_json_payload(file_entry_payload)
    included_bytes = sum(entry.bytes for entry in file_entries)
    error_count, warning_count = severity_counts(findings)

    archive_sha256: str | None = None
    archive_written = False
    should_write_archive = (not args.dry_run) and not (args.strict and error_count > 0)
    if should_write_archive:
        write_archive(
            archive_root,
            output_path,
            included,
            deterministic=not args.preserve_mtime,
            compresslevel=args.compresslevel,
            synthetic_files=synthetic_files,
            source_date_epoch=policy.source_date_epoch,
        )
        archive_sha256 = sha256_file(output_path)
        archive_written = True

    report_obj = ArchiveReport(
        script=Path(__file__).name,
        script_version=SCRIPT_VERSION,
        created_utc=utc_now_iso(),
        root=str(root),
        archive_root=str(archive_root),
        include_roots=[str(path) for path in include_roots],
        external_path_dep_roots=[str(path) for path in external_path_dep_roots],
        output=str(output_path),
        profile_requested=args.profile,
        profile_resolved=resolved_profile,
        mode=args.mode,
        package_role=policy.package_role,
        strict=args.strict,
        dry_run=args.dry_run,
        deterministic_zip_timestamps=not args.preserve_mtime,
        included_count=len(file_entries),
        included_bytes=included_bytes,
        excluded_file_count=len(excluded),
        pruned_dir_count=len(pruned_dirs),
        findings_count=len(findings),
        error_count=error_count,
        warning_count=warning_count,
        archive_sha256=archive_sha256,
        archive_zip_byte_sha256=archive_sha256,
        archive_sha256_semantics="zip-byte-sha256-not-canonical-content-hash",
        content_manifest_sha256=content_manifest_sha256,
        archive_written=archive_written,
        manifest_path=str(manifest_path) if manifest_path else None,
        report_path=str(report_path) if report_path else None,
        excluded_path=str(excluded_path) if excluded_path else None,
        findings_path=str(findings_path) if findings_path else None,
        decision_log_path=str(decision_log_path) if decision_log_path else None,
        policy_path=policy.policy_path,
        ecosystem_parity=[asdict(result) for result in ecosystem_results],
        codex_archive=codex_archive_summary(codex_archive_result),
        root_markdown_archive=root_markdown_archive_summary(root_markdown_archive_result),
        root_package_archive=root_package_archive_summary(root_package_archive_result),
    )

    result = BuildResult(
        report=report_obj,
        files=file_entries,
        excluded=excluded,
        pruned_dirs=pruned_dirs,
        findings=findings,
        decisions=decisions,
        ecosystem_parity=ecosystem_results,
    )

    manifest_payload = {
        "run": args.codex_current_run,
        "package": str(output_path),
        "manifest": str(manifest_path) if manifest_path else None,
        "excluded": str(excluded_path) if excluded_path else None,
        "findings": str(findings_path) if findings_path else None,
        "sidecars": {
            "package": str(output_path),
            "manifest": str(manifest_path) if manifest_path else None,
            "report": str(report_path) if report_path else None,
            "excluded": str(excluded_path) if excluded_path else None,
            "findings": str(findings_path) if findings_path else None,
            "decision_log": str(decision_log_path) if decision_log_path else None,
            "codex_archive_report": str(codex_archive_report_path) if codex_archive_report_path else None,
        },
        "archive_zip_byte_sha256": archive_sha256,
        "archive_sha256_semantics": "zip-byte-sha256-not-canonical-content-hash",
        "content_manifest_sha256": content_manifest_sha256,
        "report": asdict(report_obj),
        "policy": asdict(policy),
        "decisions": [asdict(entry) for entry in decisions],
        "ecosystem_parity": [asdict(entry) for entry in ecosystem_results],
        "codex_archive": asdict(codex_archive_result) if codex_archive_result else None,
        "root_markdown_archive": asdict(root_markdown_archive_result) if root_markdown_archive_result else None,
        "root_package_archive": asdict(root_package_archive_result) if root_package_archive_result else None,
        "files": file_entry_payload,
        "summaries": {
            "extensions": summarize_extensions(file_entries),
            "top_level_dirs": summarize_top_dirs(file_entries),
            "exclusion_reasons": summarize_exclusion_reasons(excluded),
        },
    }

    if manifest_path:
        write_json(manifest_path, manifest_payload)
    if excluded_path:
        write_json(excluded_path, {
            "created_utc": report_obj.created_utc,
            "root": report_obj.root,
            "excluded": [asdict(entry) for entry in excluded],
            "pruned_dirs": [asdict(entry) for entry in pruned_dirs],
            "summary": summarize_exclusion_reasons(excluded),
        })
    if findings_path:
        write_json(findings_path, {
            "created_utc": report_obj.created_utc,
            "root": report_obj.root,
            "error_count": error_count,
            "warning_count": warning_count,
            "findings": [asdict(entry) for entry in findings],
        })
    if decision_log_path:
        decision_log_path.parent.mkdir(parents=True, exist_ok=True)
        with decision_log_path.open("w", encoding="utf-8") as f:
            for decision in decisions:
                f.write(json.dumps(asdict(decision), sort_keys=True) + "\n")
    if report_path:
        markdown = render_markdown_report(
            result,
            extension_summary=summarize_extensions(file_entries),
            top_dir_summary=summarize_top_dirs(file_entries),
            exclusion_summary=summarize_exclusion_reasons(excluded),
        )
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(markdown, encoding="utf-8")

    return result


def build_package_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Create an audited source/context zip archive with manifest, report, and validation gates.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.set_defaults(command="package")
    parser.add_argument("--root", default=".", help="Workspace/repository root to archive.")
    parser.add_argument("-o", "--output", default=None, help="Output zip path. Relative paths are resolved under --root.")
    parser.add_argument("--profile", choices=PROFILES, default="auto", help="Project profile used for required-surface checks.")
    parser.add_argument("--mode", choices=MODES, default="next-codex-context", help="Archive policy mode. Legacy aliases: codex-context=next-codex-context, full-context=codex-run-full.")

    parser.add_argument("--strict", dest="strict", action="store_true", default=True, help="Exit with code 2 if validation errors are found.")
    parser.add_argument("--no-strict", dest="strict", action="store_false", help="Write archive even when validation errors are found.")
    parser.add_argument("--dry-run", action="store_true", help="Do not write the zip; still emit sidecar reports.")
    parser.add_argument("--archive-codex-runs", dest="archive_codex_runs", action="store_true", default=True, help="Normalize stale active Codex-run artifacts into docs/codex-runs/archive before packaging.")
    parser.add_argument("--no-archive-codex-runs", dest="archive_codex_runs", action="store_false", help="Diagnostic only: do not normalize stale Codex-run artifacts before packaging.")
    parser.add_argument("--archive-only", action="store_true", help="Run Codex-run archival normalization and exit without writing a zip.")
    parser.add_argument("--verify-codex-archive-hygiene", action="store_true", help="Verify no stale active Codex-run artifacts remain outside the archive/current allowlist.")
    parser.add_argument("--archive-root-markdown-noise", action="store_true", default=False, help="Archive run/audit/spec/prompt/matrix residue Markdown files found directly in workspace root.")
    parser.add_argument("--verify-root-markdown-noise-hygiene", action="store_true", help="Verify root Markdown noise hygiene without moving root Markdown files.")
    parser.add_argument("--root-markdown-archive-root", default=ROOT_MARKDOWN_ARCHIVE_DIR, help="Archive root for root Markdown noise. Relative paths are resolved under --root.")
    parser.add_argument("--root-markdown-archive-dry-run", action="store_true", help="Do not write root Markdown archive moves.")
    parser.add_argument("--include-root-markdown-archive", action="store_true", help="Include root Markdown archive directory in package collection.")
    parser.add_argument("--include-codex-archive", action="store_true", help="Include docs/codex-runs/archive history deliberately.")
    parser.add_argument("--archive-root-package-artifacts", dest="archive_root_package_artifacts", action="store_true", default=None, help="Archive root-level prior package artifacts before package collection.")
    parser.add_argument("--no-archive-root-package-artifacts", dest="archive_root_package_artifacts", action="store_false", help="Diagnostic only: do not archive root-level prior package artifacts before package collection.")
    parser.add_argument("--verify-root-package-hygiene", action="store_true", help="Verify no root-level prior package artifacts remain without moving files.")
    parser.add_argument("--root-package-archive-root", default=ROOT_PACKAGE_ARCHIVE_DIR, help="Archive root for root package artifacts. Relative paths are resolved under --root.")
    parser.add_argument("--root-package-archive-dry-run", action="store_true", help="Do not write root package artifact archive moves.")
    parser.add_argument("--include-root-package-archive", action="store_true", help="Include root package artifact archive directory in package collection.")
    parser.add_argument("--codex-current-run", default="P30", help="Current Codex run identifier allowed to remain active where explicitly permitted.")
    parser.add_argument("--codex-archive-root", default="docs/codex-runs/archive", help="Archive root for stale Codex-run artifacts. Relative paths are resolved under --root.")
    parser.add_argument("--codex-archive-report-out", default=None, help="Codex archival normalization report JSON path. Use '-' to disable. Default: <archive>.codex-archive.json")
    parser.add_argument("--policy", default=None, help="Optional PackagePolicyV1 TOML/JSON file.")

    parser.add_argument("--manifest-out", default=None, help="Manifest JSON path. Use '-' to disable. Default: <archive>.manifest.json")
    parser.add_argument("--manifest", default=None, help=argparse.SUPPRESS)
    parser.add_argument("--report-out", default=None, help="Markdown report path. Use '-' to disable. Default: <archive>.report.md")
    parser.add_argument("--excluded-out", default=None, help="Excluded file JSON path. Use '-' to disable. Default: <archive>.excluded.json")
    parser.add_argument("--findings-out", default=None, help="Findings JSON path. Use '-' to disable. Default: <archive>.findings.json")
    parser.add_argument("--decision-log-out", default=None, help="Decision provenance JSONL path. Default: <archive>.decision-log.jsonl when policy enables it.")
    parser.add_argument("--verify-package", default=None, help=argparse.SUPPRESS)

    parser.add_argument("--include-external-path-deps", dest="include_external_path_deps", action="store_true", default=True, help="Include Cargo path dependencies outside --root and store paths from their common parent.")
    parser.add_argument("--no-include-external-path-deps", dest="include_external_path_deps", action="store_false", help="Only archive files under --root; external Cargo path dependencies remain validation findings.")
    parser.add_argument("--include-generated-schemas", dest="include_generated_schemas", action="store_true", default=None, help="Include schemas.generated/ and equivalent generated schema dirs.")
    parser.add_argument("--exclude-generated-schemas", dest="include_generated_schemas", action="store_false", help="Exclude generated schema dirs.")
    parser.add_argument("--include-codex-artifacts", dest="include_codex_artifacts", action="store_true", default=None, help="Include .codex/ and equivalent Codex handoff dirs.")
    parser.add_argument("--exclude-codex-artifacts", dest="include_codex_artifacts", action="store_false", help="Exclude .codex/ and equivalent Codex handoff dirs.")
    parser.add_argument("--include-editor-config", action="store_true", help="Include .vscode/ and .idea/.")
    parser.add_argument("--include-doc-binaries", dest="include_doc_binaries", action="store_true", default=None, help="Include .pdf/.docx/.pptx/.xlsx files.")
    parser.add_argument("--exclude-doc-binaries", dest="include_doc_binaries", action="store_false", help="Exclude .pdf/.docx/.pptx/.xlsx files.")
    parser.add_argument("--include-images", dest="include_images", action="store_true", default=None, help="Include common image files.")
    parser.add_argument("--exclude-images", dest="include_images", action="store_false", help="Exclude common image files.")
    parser.add_argument("--include-logs", dest="include_logs", action="store_true", default=None, help="Include .log files.")
    parser.add_argument("--exclude-logs", dest="include_logs", action="store_false", help="Exclude .log files.")
    parser.add_argument("--allow-secret-like-names", action="store_true", help="Do not exclude files solely because their names look secret-like. Content scanning still applies if enabled.")
    parser.add_argument("--follow-symlinks", action="store_true", help="Follow symlinks if their targets remain inside root.")

    parser.add_argument("--check-rust-include-refs", action="store_true", default=True, help="Check include_str!/include_bytes! references.")
    parser.add_argument("--no-check-rust-include-refs", dest="check_rust_include_refs", action="store_false", help="Disable Rust include reference checks.")
    parser.add_argument("--check-cargo-path-deps", action="store_true", default=True, help="Check Cargo path dependencies for self-containment.")
    parser.add_argument("--no-check-cargo-path-deps", dest="check_cargo_path_deps", action="store_false", help="Disable Cargo path dependency checks.")
    parser.add_argument("--allow-external-path-deps", action="store_true", help="Downgrade external Cargo path dependencies from errors to warnings.")
    parser.add_argument("--check-script-refs", action="store_true", default=True, help="Conservatively check shell script references to .sh/.py files.")
    parser.add_argument("--no-check-script-refs", dest="check_script_refs", action="store_false", help="Disable shell script reference checks.")
    parser.add_argument("--check-secrets", action="store_true", default=True, help="Scan included text files for high-risk secret patterns.")
    parser.add_argument("--no-check-secrets", dest="check_secrets", action="store_false", help="Disable content secret scanning.")

    parser.add_argument("--max-file-size-mb", type=float, default=25.0, help="Maximum file size to include. Use 0 for no limit.")
    parser.add_argument("--secret-scan-max-kb", type=int, default=1024, help="Only scan text files up to this size for secret-like content.")
    parser.add_argument("--compresslevel", type=int, default=9, choices=range(0, 10), help="ZIP compression level 0-9.")
    parser.add_argument("--preserve-mtime", action="store_true", help="Preserve file mtimes in zip entries. Default is deterministic timestamps.")

    return parser


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    argv = list(argv)
    if argv and argv[0] == "package":
        return build_package_parser().parse_args(argv[1:])
    if argv and argv[0] == "verify":
        parser = argparse.ArgumentParser(
            description="Verify a z.py package and manifest without relying on the source tree.",
            formatter_class=argparse.ArgumentDefaultsHelpFormatter,
        )
        parser.set_defaults(command="verify")
        parser.add_argument("--package", required=True, help="Package archive to verify.")
        parser.add_argument("--manifest", required=True, help="Manifest JSON to verify against.")
        parser.add_argument("--strict", dest="strict", action="store_true", default=True)
        parser.add_argument("--no-strict", dest="strict", action="store_false")
        return parser.parse_args(argv[1:])
    if argv and argv[0] == "policy":
        parser = argparse.ArgumentParser(description="PackagePolicyV1 helpers.")
        sub = parser.add_subparsers(dest="policy_command", required=True)
        init = sub.add_parser("init", help="Write a starter zpy.package.toml policy.")
        init.set_defaults(command="policy")
        init.add_argument("--root", default=".")
        init.add_argument("--policy", default="zpy.package.toml")
        init.add_argument("--force", action="store_true")
        validate = sub.add_parser("validate", help="Validate a PackagePolicyV1 TOML/JSON policy.")
        validate.set_defaults(command="policy")
        validate.add_argument("--policy", required=True)
        return parser.parse_args(argv[1:])
    if argv and argv[0] == "explain":
        parser = argparse.ArgumentParser(description="Explain the include/exclude decision for a manifest path.")
        parser.set_defaults(command="explain")
        parser.add_argument("--manifest", required=True)
        parser.add_argument("--path", required=True)
        return parser.parse_args(argv[1:])
    if argv and argv[0] == "compare":
        parser = argparse.ArgumentParser(description="Compare two z.py manifests.")
        parser.set_defaults(command="compare")
        parser.add_argument("--old", required=True)
        parser.add_argument("--new", required=True)
        return parser.parse_args(argv[1:])

    args = build_package_parser().parse_args(argv)
    if args.verify_package:
        args.command = "verify"
        args.package = args.verify_package
        if not args.manifest:
            raise SystemExit("--verify-package requires --manifest")
    return args


def print_console_summary(result: BuildResult) -> None:
    r = result.report
    status = "FAILED" if r.error_count else "OK"
    print(f"[{status}] profile={r.profile_resolved} mode={r.mode} role={r.package_role} included={r.included_count} bytes={r.included_bytes}")
    if r.codex_archive:
        codex = r.codex_archive
        print(
            "codex_archive: "
            f"enabled={codex.get('enabled')} planned={codex.get('planned_count')} "
            f"moved={codex.get('moved_count')} active_after={codex.get('active_stale_after_count')}"
        )
        if codex.get("report_path"):
            print(f"codex_archive_report: {codex.get('report_path')}")
    if r.root_markdown_archive:
        root_md = r.root_markdown_archive
        print(
            "root_markdown_archive: "
            f"enabled={root_md.get('enabled')} inspected={root_md.get('inspected_count')} "
            f"candidates={root_md.get('candidate_count')} ambiguous={root_md.get('ambiguous_count')} "
            f"moved={root_md.get('moved_count')} collisions={root_md.get('collision_count')}"
        )
        if root_md.get("manifest_path"):
            print(f"root_markdown_archive_manifest: {root_md.get('manifest_path')}")
    if r.root_package_archive:
        root_pkg = r.root_package_archive
        print(
            "root_package_archive: "
            f"enabled={root_pkg.get('enabled')} inspected={root_pkg.get('inspected_count')} "
            f"candidates={root_pkg.get('candidate_count')} moved={root_pkg.get('moved_count')} "
            f"skipped_existing={root_pkg.get('skipped_existing_count')} collisions={root_pkg.get('collision_count')}"
        )
        if root_pkg.get("manifest_path"):
            print(f"root_package_archive_manifest: {root_pkg.get('manifest_path')}")
    if r.archive_root != r.root:
        print(f"archive_root: {r.archive_root}")
        print(f"include_roots: {len(r.include_roots)} ({len(r.external_path_dep_roots)} external Cargo path deps)")
    if r.archive_written:
        print(f"wrote: {r.output}")
        if r.archive_sha256:
            print(f"zip-byte-sha256: {r.archive_zip_byte_sha256}")
            print(f"archive-hash-semantics: {r.archive_sha256_semantics}")
        if r.content_manifest_sha256:
            print(f"content-manifest-sha256: {r.content_manifest_sha256}")
    elif r.codex_archive and r.codex_archive.get("archive_only"):
        print("archive-only: zip not written")
    elif r.codex_archive and r.codex_archive.get("verify_only"):
        print("verify-codex-archive-hygiene: zip not written")
    elif r.dry_run:
        print("dry-run: zip not written")
    else:
        print("zip not written because strict validation errors were found")
    if r.manifest_path:
        print(f"manifest: {r.manifest_path}")
    if r.report_path:
        print(f"report: {r.report_path}")
    if r.findings_count:
        print(f"findings: {r.findings_count} ({r.error_count} errors, {r.warning_count} warnings)")
        for finding in result.findings[:20]:
            print(f"  - {finding.severity.upper()} {finding.code} {finding.path}: {finding.detail}")
        if len(result.findings) > 20:
            print(f"  ... {len(result.findings) - 20} more; see findings JSON/report")


def verify_package(package_path: Path, manifest_path: Path) -> tuple[list[Finding], dict[str, Any]]:
    findings: list[Finding] = []
    payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest_files = {
        item["path"]: item
        for item in payload.get("files", [])
        if isinstance(item, dict) and isinstance(item.get("path"), str)
    }
    if not package_path.exists():
        findings.append(Finding("verify-package-missing", "error", str(package_path), "Package archive does not exist."))
        return findings, payload
    seen_zip_names: set[str] = set()
    try:
        with zipfile.ZipFile(package_path) as zf:
            names = zf.namelist()
            for name in names:
                if name in seen_zip_names:
                    findings.append(Finding("verify-duplicate-archive-entry", "error", name, "Duplicate archive entry name."))
                seen_zip_names.add(name)
                if not is_safe_archive_name(name):
                    findings.append(Finding("verify-unsafe-archive-entry", "error", name, "Archive entry path is not transfer-safe."))
            zip_names = set(names)
            for rel, item in manifest_files.items():
                if rel not in zip_names:
                    findings.append(Finding("verify-manifest-file-missing-in-package", "error", rel, "Manifest file entry is absent from the zip."))
                    continue
                with zf.open(rel) as f:
                    digest = hashlib.sha256(f.read()).hexdigest()
                if digest != item.get("sha256"):
                    findings.append(Finding("verify-manifest-hash-mismatch", "error", rel, "Zip entry SHA-256 does not match manifest."))
            extra = sorted(zip_names - set(manifest_files))
            for rel in extra[:50]:
                findings.append(Finding("verify-package-entry-missing-from-manifest", "error", rel, "Zip contains an entry not declared in manifest files."))
            if len(extra) > 50:
                findings.append(Finding("verify-package-entry-missing-from-manifest-truncated", "error", "/", f"{len(extra) - 50} additional undeclared zip entries omitted."))
    except zipfile.BadZipFile as exc:
        findings.append(Finding("verify-bad-zip", "error", str(package_path), str(exc)))
    expected_sha = payload.get("archive_zip_byte_sha256") or payload.get("archive_sha256")
    if expected_sha and package_path.exists():
        actual_sha = sha256_file(package_path)
        if actual_sha != expected_sha:
            findings.append(Finding("verify-archive-sha256-mismatch", "error", str(package_path), "Archive byte SHA-256 does not match manifest."))
    return findings, payload


def run_verify_command(args: argparse.Namespace) -> int:
    findings, _payload = verify_package(Path(args.package).expanduser().resolve(), Path(args.manifest).expanduser().resolve())
    errors, warnings = severity_counts(findings)
    if findings:
        print(f"verify: {len(findings)} findings ({errors} errors, {warnings} warnings)")
        for finding in findings[:50]:
            print(f"  - {finding.severity.upper()} {finding.code} {finding.path}: {finding.detail}")
    else:
        print("verify: OK")
    return 2 if args.strict and errors else 0


def starter_policy(root: Path) -> str:
    package_name = root.name or "package"
    return "\n".join([
        'schema = "PackagePolicyV1"',
        "",
        "[package]",
        f'name = "{package_name}"',
        'role = "source-context"',
        'root = "."',
        "",
        "[modes.next-codex-context]",
        "include_logs = false",
        "include_patch_artifacts = true",
        "include_codex_artifacts = false",
        "",
        "[ecosystem_parity]",
        "enabled = true",
        'default_severity = "info"',
        "",
        "[security]",
        "check_secrets = true",
        "allow_secret_like_names = false",
        "follow_symlinks = false",
        "fail_on_unicode_collision = true",
        "fail_on_case_collision = false",
        "fail_on_windows_reserved_name = true",
        "",
        "[archive]",
        "deterministic_timestamps = true",
        "emit_decision_log = false",
        "",
    ])


def run_policy_command(args: argparse.Namespace) -> int:
    if args.policy_command == "init":
        root = Path(args.root).expanduser().resolve()
        path = Path(args.policy).expanduser()
        if not path.is_absolute():
            path = root / path
        if path.exists() and not args.force:
            print(f"policy init refused to overwrite existing file: {path}", file=sys.stderr)
            return 2
        path.write_text(starter_policy(root), encoding="utf-8")
        print(f"wrote policy: {path}")
        return 0
    path = Path(args.policy).expanduser().resolve()
    try:
        payload = read_policy_document(path)
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        print(f"policy invalid: {exc}", file=sys.stderr)
        return 2
    errors = validate_policy_document(payload)
    if errors:
        print("policy invalid:")
        for error in errors:
            print(f"  - {error}")
        return 2
    print(f"policy OK: {path}")
    return 0


def run_explain_command(args: argparse.Namespace) -> int:
    payload = json.loads(Path(args.manifest).expanduser().read_text(encoding="utf-8"))
    wanted = args.path.strip("/")
    for decision in payload.get("decisions", []):
        if decision.get("path") == wanted:
            print(json.dumps(decision, indent=2, sort_keys=True))
            return 0
    for item in payload.get("files", []):
        if item.get("path") == wanted:
            print(json.dumps({
                "path": wanted,
                "decision": "include",
                "reason": "manifest-file-entry",
                "source": "manifest",
                "mode": payload.get("report", {}).get("mode"),
            }, indent=2, sort_keys=True))
            return 0
    print(f"no decision recorded for {wanted}", file=sys.stderr)
    return 2


def run_compare_command(args: argparse.Namespace) -> int:
    old = json.loads(Path(args.old).expanduser().read_text(encoding="utf-8"))
    new = json.loads(Path(args.new).expanduser().read_text(encoding="utf-8"))
    old_files = {item["path"]: item for item in old.get("files", []) if isinstance(item, dict) and "path" in item}
    new_files = {item["path"]: item for item in new.get("files", []) if isinstance(item, dict) and "path" in item}
    added = sorted(set(new_files) - set(old_files))
    removed = sorted(set(old_files) - set(new_files))
    changed = sorted(path for path in set(old_files) & set(new_files) if old_files[path].get("sha256") != new_files[path].get("sha256"))
    print(json.dumps({
        "added": added,
        "removed": removed,
        "changed": changed,
        "summary": {
            "added_count": len(added),
            "removed_count": len(removed),
            "changed_count": len(changed),
        },
    }, indent=2, sort_keys=True))
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        if args.command == "verify":
            return run_verify_command(args)
        if args.command == "policy":
            return run_policy_command(args)
        if args.command == "explain":
            return run_explain_command(args)
        if args.command == "compare":
            return run_compare_command(args)
        result = build(args)
        print_console_summary(result)
        if args.strict and result.report.error_count:
            return 2
        return 0
    except KeyboardInterrupt:
        print("interrupted", file=sys.stderr)
        return 130
    except Exception as exc:  # pragma: no cover - operational guardrail
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
