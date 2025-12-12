# Extension Plan - Adding to Existing Structure

## ✅ What Already Exists (We Keep This!)

### 1. Database Schema (Migration 011)
```sql
-- ✅ ALREADY EXISTS - We keep this
CREATE TABLE modules (
    id UUID PRIMARY KEY,
    name VARCHAR(100) UNIQUE,
    display_name VARCHAR(255),
    description TEXT,
    category VARCHAR(50), -- 'core', 'operational', 'integration'
    enabled_by_default BOOLEAN,
    requires_license BOOLEAN
);

-- ✅ ALREADY EXISTS - We keep this
CREATE TABLE module_activations (
    id UUID PRIMARY KEY,
    module_id UUID REFERENCES modules(id),
    enabled BOOLEAN,
    activated_at TIMESTAMP,
    deactivated_at TIMESTAMP
);

-- ✅ ALREADY EXISTS - We keep this
CREATE FUNCTION is_module_enabled(module_name VARCHAR) RETURNS BOOLEAN;
```

### 2. Module Service (src/module_service.rs)
```rust
// ✅ ALREADY EXISTS - We keep this
pub struct ModuleService {
    pool: PgPool,
    cache: HashMap<String, bool>,
}

// ✅ ALREADY EXISTS - We keep this
impl ModuleService {
    pub async fn is_module_enabled(&mut self, module_name: &str) -> Result<bool, sqlx::Error>
    pub async fn enable_module(&mut self, module_name: &str, ...) -> Result<(), sqlx::Error>
    pub async fn disable_module(&mut self, module_name: &str, ...) -> Result<(), sqlx::Error>
    pub async fn get_all_modules(&self) -> Result<Vec<(ModuleDb, bool)>, sqlx::Error>
}
```

### 3. Existing Modules (src/modules/)
```
✅ ALREADY EXISTS - We keep all of these:
- data_subject_rights.rs
- human_oversight.rs
- risk_assessment.rs
- breach_management.rs
- consent.rs
- dpia.rs
- retention.rs
- monitoring.rs
- green_ai.rs
- ai_bom.rs
```

### 4. Wizard System (src/services/wizard_service.rs)
```rust
// ✅ ALREADY EXISTS - We keep this
pub struct WizardService {
    pool: PgPool,
}

// ✅ ALREADY EXISTS - We keep this
impl WizardService {
    pub async fn create_or_update_company_profile(...)
    pub async fn get_recommended_modules(...)
    pub async fn calculate_pricing(...)
    pub async fn start_trial(...)
}
```

### 5. API Endpoints (src/routes/modules.rs)
```rust
// ✅ ALREADY EXISTS - We keep this
GET  /api/v1/modules
POST /api/v1/modules/{name}/enable
POST /api/v1/modules/{name}/disable
GET  /api/v1/modules/{name}/status
```

---

## 🆕 What We're ADDING (Not Replacing!)

### 1. Enhanced Database Schema (NEW Migration)

**We ADD columns to existing tables:**

```sql
-- Migration: 035_enhanced_module_system.sql

-- ✅ ADD columns to existing modules table (doesn't break existing data)
ALTER TABLE modules ADD COLUMN IF NOT EXISTS tier INTEGER DEFAULT 2; -- 1=Core, 2=Regulatory, 3=Operational
ALTER TABLE modules ADD COLUMN IF NOT EXISTS regulation VARCHAR(50);
ALTER TABLE modules ADD COLUMN IF NOT EXISTS article_number VARCHAR(20);
ALTER TABLE modules ADD COLUMN IF NOT EXISTS dependencies TEXT[];
ALTER TABLE modules ADD COLUMN IF NOT EXISTS conflicts TEXT[];
ALTER TABLE modules ADD COLUMN IF NOT EXISTS auto_enable_conditions JSONB;
ALTER TABLE modules ADD COLUMN IF NOT EXISTS configuration_schema JSONB;
ALTER TABLE modules ADD COLUMN IF NOT EXISTS base_price_monthly DECIMAL(10,2);
ALTER TABLE modules ADD COLUMN IF NOT EXISTS per_system_price_monthly DECIMAL(10,2);

-- ✅ NEW table for module-regulation mapping (adds to existing structure)
CREATE TABLE IF NOT EXISTS module_regulation_mapping (
    module_id UUID REFERENCES modules(id),
    regulation VARCHAR(50) NOT NULL,
    article_number VARCHAR(20),
    requirement_level VARCHAR(20), -- 'MANDATORY', 'RECOMMENDED', 'OPTIONAL'
    PRIMARY KEY (module_id, regulation, article_number)
);

-- ✅ NEW table for per-company module configurations (adds to existing structure)
CREATE TABLE IF NOT EXISTS company_module_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID REFERENCES company_profiles(id),
    module_id UUID REFERENCES modules(id),
    enabled BOOLEAN DEFAULT false,
    configuration JSONB DEFAULT '{}'::jsonb,
    configured_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(company_id, module_id)
);

-- ✅ NEW table for policy templates (adds to existing structure)
CREATE TABLE IF NOT EXISTS policy_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    regulation VARCHAR(50) NOT NULL,
    article_number VARCHAR(20),
    template_type VARCHAR(50),
    policy_config JSONB NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**Key Point:** All existing modules continue to work! We just add more metadata.

---

### 2. Enhanced Module Service (EXTEND, Not Replace)

**We ADD methods to existing ModuleService:**

```rust
// src/module_service.rs

// ✅ KEEP existing methods
impl ModuleService {
    // ... existing methods stay the same ...
    
    // 🆕 ADD new methods (don't break existing code)
    
    /// Get module configuration for a company
    pub async fn get_company_module_config(
        &self,
        company_id: Uuid,
        module_name: &str,
    ) -> Result<Option<serde_json::Value>, sqlx::Error> {
        // NEW functionality
    }
    
    /// Set module configuration for a company
    pub async fn set_company_module_config(
        &self,
        company_id: Uuid,
        module_name: &str,
        config: serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        // NEW functionality
    }
    
    /// Get modules by regulation
    pub async fn get_modules_by_regulation(
        &self,
        regulation: &str,
    ) -> Result<Vec<ModuleDb>, sqlx::Error> {
        // NEW functionality
    }
}
```

**Key Point:** All existing code continues to work! We just add new methods.

---

### 3. New Module Files (ADD to existing modules/)

**We ADD new module files, existing ones stay:**

```
src/modules/
├── ✅ data_subject_rights.rs (KEEP - no changes)
├── ✅ human_oversight.rs (KEEP - no changes)
├── ✅ risk_assessment.rs (KEEP - no changes)
├── ... (all existing modules stay)
│
├── 🆕 gdpr/
│   ├── article_28_processor_obligations.rs (NEW)
│   ├── article_44_49_international_transfers.rs (NEW)
│   └── article_31_supervisory_authority.rs (NEW)
│
├── 🆕 eu_ai_act/
│   ├── article_15_accuracy_robustness.rs (NEW)
│   ├── article_19_foundation_models.rs (NEW)
│   └── article_44_50_prohibited_practices.rs (NEW)
│
├── 🆕 dora/
│   ├── article_4_8_ict_risk_management.rs (NEW)
│   └── article_17_20_resilience_testing.rs (NEW)
│
├── 🆕 financial/
│   ├── psd2_sca.rs (NEW)
│   ├── mica_compliance.rs (NEW)
│   └── mifid_ii.rs (NEW)
│
└── 🆕 healthcare/
    ├── mdr_compliance.rs (NEW)
    └── ivdr_compliance.rs (NEW)
```

**Key Point:** Existing modules untouched! We just add new ones.

---

### 4. Enhanced Wizard Service (EXTEND, Not Replace)

**We ADD methods to existing WizardService:**

```rust
// src/services/wizard_service.rs

// ✅ KEEP existing methods
impl WizardService {
    // ... existing methods stay the same ...
    
    // 🆕 ADD new methods
    
    /// Get recommended modules with enhanced logic
    pub async fn get_recommended_modules_enhanced(
        &self,
        industry: &str,
        country: &str,
        regulatory_requirements: &[String],
        ai_use_cases: &[String],
    ) -> Result<ModuleRecommendationResponse, sqlx::Error> {
        // Enhanced recommendation logic
    }
    
    /// Apply policy templates to company
    pub async fn apply_policy_templates(
        &self,
        company_id: Uuid,
        profile: &CompanyProfile,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        // NEW functionality
    }
    
    /// Configure modules for company
    pub async fn configure_modules(
        &self,
        company_id: Uuid,
        module_configs: Vec<ModuleConfig>,
    ) -> Result<(), sqlx::Error> {
        // NEW functionality
    }
}
```

**Key Point:** Existing wizard flow continues to work! We just add new steps.

---

### 5. Enhanced API Endpoints (ADD to existing routes)

**We ADD new endpoints, existing ones stay:**

```rust
// src/routes/modules.rs

// ✅ KEEP existing endpoints
GET  /api/v1/modules
POST /api/v1/modules/{name}/enable
POST /api/v1/modules/{name}/disable
GET  /api/v1/modules/{name}/status

// 🆕 ADD new endpoints
GET  /api/v1/modules/by-regulation/{regulation}
GET  /api/v1/modules/{name}/config-schema
POST /api/v1/companies/{company_id}/modules/{module_name}/config
GET  /api/v1/companies/{company_id}/modules/{module_name}/config
GET  /api/v1/policy-templates
POST /api/v1/companies/{company_id}/apply-templates
```

**Key Point:** All existing endpoints continue to work! We just add new ones.

---

## 📊 Migration Strategy

### Step 1: Database Migration (Non-Breaking)
```sql
-- Migration 035: Add new columns (all nullable, so existing data works)
ALTER TABLE modules ADD COLUMN IF NOT EXISTS tier INTEGER;
ALTER TABLE modules ADD COLUMN IF NOT EXISTS regulation VARCHAR(50);
-- ... etc

-- Backfill existing modules with default values
UPDATE modules SET tier = 2 WHERE tier IS NULL; -- Existing modules = Tier 2
UPDATE modules SET category = 'operational' WHERE category IS NULL;
```

### Step 2: Add New Module Files
```rust
// Just create new files, don't touch existing ones
src/modules/gdpr/article_28_processor_obligations.rs
src/modules/gdpr/article_44_49_international_transfers.rs
// ... etc
```

### Step 3: Register New Modules in Database
```sql
-- Insert new modules (existing ones already there)
INSERT INTO modules (name, display_name, category, tier, regulation, article_number) VALUES
('gdpr_article_28', 'GDPR Article 28 - Processor Obligations', 'regulatory', 2, 'GDPR', 'Article 28'),
('gdpr_article_44_49', 'GDPR Article 44-49 - International Transfers', 'regulatory', 2, 'GDPR', 'Article 44-49'),
-- ... etc
```

### Step 4: Extend Services (Add Methods)
```rust
// Just add new methods to existing services
impl ModuleService {
    // ... existing methods ...
    
    // Add new methods here
}
```

### Step 5: Add New API Endpoints
```rust
// Add new routes to existing router
.route("/modules/by-regulation/{regulation}", ...)
.route("/companies/{company_id}/modules/{module_name}/config", ...)
```

---

## ✅ Compatibility Guarantee

### What Won't Break:
1. ✅ **Existing modules** - All continue to work
2. ✅ **Existing API endpoints** - All continue to work
3. ✅ **Existing wizard flow** - Still works (we just add steps)
4. ✅ **Existing database data** - All preserved
5. ✅ **Existing code** - No breaking changes

### What We Add:
1. 🆕 **New modules** - Just more modules
2. 🆕 **New API endpoints** - Just more endpoints
3. 🆕 **New wizard steps** - Just more steps
4. 🆕 **New database columns** - All nullable, so existing data works
5. 🆕 **New database tables** - Don't affect existing tables

---

## 🎯 Implementation Order

### Week 1: Database Extension
1. ✅ Create migration 035 (adds columns, new tables)
2. ✅ Backfill existing modules with metadata
3. ✅ Test: All existing queries still work

### Week 2: Service Extension
1. ✅ Add new methods to ModuleService
2. ✅ Add new methods to WizardService
3. ✅ Test: All existing methods still work

### Week 3: New Modules
1. ✅ Create first batch of new modules (GDPR Article 28, 44-49)
2. ✅ Register in database
3. ✅ Test: New modules work, existing modules unaffected

### Week 4: API Extension
1. ✅ Add new API endpoints
2. ✅ Test: New endpoints work, existing endpoints unaffected

### Week 5: Wizard Enhancement
1. ✅ Add new wizard steps
2. ✅ Test: New steps work, existing wizard flow still works

---

## 🚀 Summary

**We're EXTENDING, not replacing:**

1. ✅ **Database** - Add columns/tables (existing data preserved)
2. ✅ **Services** - Add methods (existing methods preserved)
3. ✅ **Modules** - Add new files (existing files preserved)
4. ✅ **API** - Add endpoints (existing endpoints preserved)
5. ✅ **Wizard** - Add steps (existing steps preserved)

**Everything that works now will continue to work!**

**We're just adding more capabilities on top of the solid foundation you already have.**

