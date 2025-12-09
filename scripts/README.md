# Scripts

## add-license-headers

Adds AGPL-3.0 license headers to all Rust and Python source files.

### Usage

**Windows (PowerShell):**
```powershell
.\scripts\add-license-headers.ps1
```

**Linux/Mac (Bash):**
```bash
chmod +x scripts/add-license-headers.sh
./scripts/add-license-headers.sh
```

### What it does

- Scans all `.rs` (Rust) and `.py` (Python) files in the repository
- Adds the following header to files that don't already have one:

**Rust files:**
```
// Copyright (c) 2025 Veridion Nexus.
// Licensed under the AGPL-3.0 license.
```

**Python files:**
```
# Copyright (c) 2025 Veridion Nexus.
# Licensed under the AGPL-3.0 license.
```

### Safety features

- **Skips files that already have license headers** (checks for "Copyright" and "AGPL" in first 5 lines)
- **Excludes build directories**: `target/`, `__pycache__/`, `venv/`, `env/`, `.git/`
- **Non-destructive**: Only adds headers, doesn't modify existing content

### When to run

- **Before signing contracts** with enterprise customers
- **Before major releases** to ensure legal compliance
- **After adding new source files** (run periodically)

### Important notes

- This script modifies source files. **Commit your changes** before running, or run it in a clean git state so you can review the changes.
- Review the changes with `git diff` before committing
- The script is idempotent - safe to run multiple times

