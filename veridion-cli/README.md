# Veridion CLI

Command-line tool for testing, simulating, and managing policies in Veridion Nexus.

## Installation

```bash
cd veridion-cli
cargo build --release
```

The binary will be at `target/release/veridion` (or `target/debug/veridion` for debug builds).

## Usage

### Set API Key

```bash
export VERIDION_API_KEY=your_api_key_here
```

Or use `--api-key` flag with each command.

### Test Policy

Test a policy configuration to see what would happen:

```bash
veridion test --policy-type SOVEREIGN_LOCK --config '{"blocked_countries":["US","CN"]}'
```

### Simulate Policy Impact

Simulate policy impact over a time period:

```bash
veridion simulate --policy-type SOVEREIGN_LOCK --config '{"blocked_countries":["US","CN"]}' --days 30
```

### Rollback Policy

Rollback a policy to a previous version:

```bash
# Dry-run first
veridion rollback --policy-id <uuid> --dry-run

# Actual rollback
veridion rollback --policy-id <uuid> --version 2 --notes "Reverting due to issues"
```

### Check Policy Health

Get policy health status:

```bash
veridion health --policy-id <uuid>
```

### View Shadow Mode Analytics

View shadow mode analytics:

```bash
veridion shadow --days 7
```

## Examples

```bash
# Test a policy
veridion test --policy-type SOVEREIGN_LOCK --config '{"blocked_countries":["US","CN","RU"]}'

# Simulate over 30 days
veridion simulate --policy-type SOVEREIGN_LOCK --config '{"blocked_countries":["US"]}' --days 30

# Test rollback before executing
veridion rollback --policy-id abc123 --dry-run

# Execute rollback
veridion rollback --policy-id abc123 --version 1 --notes "Production issue"
```

