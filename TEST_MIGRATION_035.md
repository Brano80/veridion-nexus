# Test Plan: Migration 035 - Enhanced Module System

## Prerequisites

1. **Database Access**: PostgreSQL database with existing schema
2. **Backup**: Create a backup before running migration
3. **API Server**: Should be stopped during migration (or use transaction)

## Test Steps

### Step 1: Pre-Migration Verification

```sql
-- Check current modules table structure
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_name = 'modules'
ORDER BY ordinal_position;

-- Count existing modules
SELECT COUNT(*) as module_count FROM modules;

-- Check existing module_activations
SELECT COUNT(*) as activation_count FROM module_activations;
```

**Expected:** Should see existing columns (id, name, display_name, etc.) but NOT new columns (tier, regulation, etc.)

---

### Step 2: Run Migration

```bash
# Option 1: Using psql
psql -d veridion_nexus -f migrations/035_enhanced_module_system.sql

# Option 2: Using docker exec (if using docker-compose)
docker-compose exec postgres psql -U postgres -d veridion_nexus -f /migrations/035_enhanced_module_system.sql
```

**Expected:** Migration runs without errors, all statements succeed

---

### Step 3: Post-Migration Verification

#### 3.1 Check New Columns Exist

```sql
-- Verify new columns were added
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_name = 'modules'
AND column_name IN ('tier', 'regulation', 'article_number', 'dependencies', 'conflicts', 
                    'auto_enable_conditions', 'configuration_schema', 
                    'base_price_monthly', 'per_system_price_monthly')
ORDER BY column_name;
```

**Expected:** All 9 new columns should exist and be nullable

#### 3.2 Check New Tables Exist

```sql
-- Check new tables
SELECT table_name 
FROM information_schema.tables 
WHERE table_name IN (
    'module_regulation_mapping',
    'company_module_configs',
    'policy_templates',
    'industry_module_recommendations',
    'use_case_module_recommendations'
)
ORDER BY table_name;
```

**Expected:** All 5 new tables should exist

#### 3.3 Check New Functions Exist

```sql
-- Check new functions
SELECT routine_name, routine_type
FROM information_schema.routines
WHERE routine_name IN (
    'get_modules_by_regulation',
    'get_company_module_config',
    'is_module_enabled_for_company'
)
ORDER BY routine_name;
```

**Expected:** All 3 new functions should exist

#### 3.4 Verify Backfill Worked

```sql
-- Check tier was set for existing modules
SELECT category, tier, COUNT(*) as count
FROM modules
GROUP BY category, tier
ORDER BY category, tier;
```

**Expected:** 
- Core modules should have tier = 1
- Operational modules should have tier = 2
- Integration modules should have tier = 3

```sql
-- Check pricing defaults
SELECT 
    COUNT(*) as total_modules,
    COUNT(base_price_monthly) as with_base_price,
    COUNT(per_system_price_monthly) as with_per_system_price
FROM modules;
```

**Expected:** All modules should have pricing set (default 0)

#### 3.5 Verify Regulation Mappings

```sql
-- Check regulation mappings were created
SELECT 
    m.name as module_name,
    mrm.regulation,
    mrm.article_number,
    mrm.requirement_level
FROM module_regulation_mapping mrm
JOIN modules m ON mrm.module_id = m.id
ORDER BY mrm.regulation, mrm.requirement_level, m.name;
```

**Expected:** Should see mappings for existing modules (GDPR, EU_AI_ACT, EIDAS)

---

### Step 4: Test New Functions

#### 4.1 Test get_modules_by_regulation()

```sql
-- Test getting GDPR modules
SELECT * FROM get_modules_by_regulation('GDPR');
```

**Expected:** Returns modules mapped to GDPR with their article numbers

#### 4.2 Test get_company_module_config()

```sql
-- First, get a company_id (if you have one)
SELECT id FROM company_profiles LIMIT 1;

-- Then test (replace with actual company_id)
SELECT get_company_module_config(
    '00000000-0000-0000-0000-000000000000'::uuid,  -- Replace with real UUID
    'module_data_subject_rights'
);
```

**Expected:** Returns empty JSON object `{}` if no config exists, or the config if it does

#### 4.3 Test is_module_enabled_for_company()

```sql
-- Test (replace with actual company_id)
SELECT is_module_enabled_for_company(
    '00000000-0000-0000-0000-000000000000'::uuid,  -- Replace with real UUID
    'module_data_subject_rights'
);
```

**Expected:** Returns `false` if not enabled, `true` if enabled

---

### Step 5: Test ModuleService Methods (via API)

#### 5.1 Start API Server

```bash
# Make sure API server is running
cargo run
# or
docker-compose up
```

#### 5.2 Test get_modules_by_regulation (when endpoint exists)

```bash
# This endpoint needs to be implemented first
curl http://localhost:8080/api/v1/modules/by-regulation/GDPR
```

**Expected:** Returns list of GDPR modules

#### 5.3 Test company module config (when endpoints exist)

```bash
# Set config
curl -X POST http://localhost:8080/api/v1/companies/{company_id}/modules/module_data_subject_rights/config \
  -H "Content-Type: application/json" \
  -d '{"require_dpa": true, "audit_frequency": "quarterly"}'

# Get config
curl http://localhost:8080/api/v1/companies/{company_id}/modules/module_data_subject_rights/config
```

**Expected:** Config is set and retrieved successfully

---

### Step 6: Verify Existing Functionality Still Works

#### 6.1 Test Existing Module Endpoints

```bash
# Get all modules
curl http://localhost:8080/api/v1/modules

# Get module status
curl http://localhost:8080/api/v1/modules/module_data_subject_rights/status

# Enable module
curl -X POST http://localhost:8080/api/v1/modules/module_data_subject_rights/enable

# Disable module
curl -X POST http://localhost:8080/api/v1/modules/module_data_subject_rights/disable
```

**Expected:** All existing endpoints continue to work

#### 6.2 Test Existing ModuleService Methods

The existing methods should still work:
- `is_module_enabled()` - Should work as before
- `enable_module()` - Should work as before
- `disable_module()` - Should work as before
- `get_all_modules()` - Should work as before

---

## Rollback Plan (if needed)

If migration fails or causes issues:

```sql
-- Rollback: Remove new columns (if needed)
ALTER TABLE modules 
    DROP COLUMN IF EXISTS tier,
    DROP COLUMN IF EXISTS regulation,
    DROP COLUMN IF EXISTS article_number,
    DROP COLUMN IF EXISTS dependencies,
    DROP COLUMN IF EXISTS conflicts,
    DROP COLUMN IF EXISTS auto_enable_conditions,
    DROP COLUMN IF EXISTS configuration_schema,
    DROP COLUMN IF EXISTS base_price_monthly,
    DROP COLUMN IF EXISTS per_system_price_monthly;

-- Drop new tables
DROP TABLE IF EXISTS use_case_module_recommendations;
DROP TABLE IF EXISTS industry_module_recommendations;
DROP TABLE IF EXISTS policy_templates;
DROP TABLE IF EXISTS company_module_configs;
DROP TABLE IF EXISTS module_regulation_mapping;

-- Drop new functions
DROP FUNCTION IF EXISTS is_module_enabled_for_company(UUID, VARCHAR);
DROP FUNCTION IF EXISTS get_company_module_config(UUID, VARCHAR);
DROP FUNCTION IF EXISTS get_modules_by_regulation(VARCHAR);

-- Drop triggers
DROP TRIGGER IF EXISTS trigger_update_policy_templates_updated_at ON policy_templates;
DROP TRIGGER IF EXISTS trigger_update_company_module_configs_updated_at ON company_module_configs;
DROP FUNCTION IF EXISTS update_policy_templates_updated_at();
DROP FUNCTION IF EXISTS update_company_module_configs_updated_at();
```

---

## Success Criteria

✅ Migration runs without errors  
✅ All new columns exist and are nullable  
✅ All new tables exist  
✅ All new functions exist  
✅ Backfill worked (tier set, pricing set)  
✅ Regulation mappings created  
✅ New functions work correctly  
✅ Existing functionality still works  
✅ No data loss  

---

## Notes

- Migration is **non-breaking** - all new columns are nullable
- Existing modules continue to work
- Existing queries continue to work
- Can be run on production (with backup first)
- Migration is idempotent (can be run multiple times safely)

