use sqlx::PgPool;
use crate::models::db_models::ModuleDb;
use uuid::Uuid;
use std::collections::HashMap;

/// Service for managing modules and feature flags
pub struct ModuleService {
    pool: PgPool,
    cache: HashMap<String, bool>,
}

impl ModuleService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cache: HashMap::new(),
        }
    }

    /// Check if a module is enabled
    pub async fn is_module_enabled(&mut self, module_name: &str) -> Result<bool, sqlx::Error> {
        // Check cache first
        if let Some(&enabled) = self.cache.get(module_name) {
            return Ok(enabled);
        }

        // Check database
        let result: Option<bool> = sqlx::query_scalar(
            "SELECT is_module_enabled($1)"
        )
        .bind(module_name)
        .fetch_optional(&self.pool)
        .await?;

        let enabled = result.unwrap_or(false);
        self.cache.insert(module_name.to_string(), enabled);
        Ok(enabled)
    }

    /// Check if a feature flag is enabled
    #[allow(dead_code)]
    pub async fn is_feature_enabled(&mut self, feature_name: &str) -> Result<bool, sqlx::Error> {
        let result: Option<bool> = sqlx::query_scalar(
            "SELECT is_feature_enabled($1)"
        )
        .bind(feature_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.unwrap_or(false))
    }

    /// Get all modules with their activation status
    pub async fn get_all_modules(&self) -> Result<Vec<(ModuleDb, bool)>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT 
                m.id, m.name, m.display_name, m.description, m.category, 
                m.enabled_by_default, m.requires_license, m.created_at, m.updated_at,
                COALESCE(ma.enabled, m.enabled_by_default) as enabled
            FROM modules m
            LEFT JOIN module_activations ma ON m.id = ma.module_id
            ORDER BY m.category, m.name
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        use sqlx::Row;
        let mut result = Vec::new();
        for row in rows {
            let module = ModuleDb {
                id: row.get::<Uuid, _>(0),
                name: row.get::<String, _>(1),
                display_name: row.get::<String, _>(2),
                description: row.get::<Option<String>, _>(3),
                category: row.get::<String, _>(4),
                enabled_by_default: row.get::<bool, _>(5),
                requires_license: row.get::<bool, _>(6),
                created_at: row.get::<chrono::DateTime<chrono::Utc>, _>(7),
                updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>(8),
            };
            let enabled: bool = row.get::<bool, _>(9);
            result.push((module, enabled));
        }

        Ok(result)
    }

    /// Enable a module
    pub async fn enable_module(
        &mut self,
        module_name: &str,
        activated_by: Option<Uuid>,
        notes: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO module_activations (module_id, enabled, activated_at, activated_by, notes)
            SELECT id, true, CURRENT_TIMESTAMP, $2, $3
            FROM modules
            WHERE name = $1
            ON CONFLICT (module_id) 
            DO UPDATE SET 
                enabled = true,
                activated_at = CURRENT_TIMESTAMP,
                activated_by = $2,
                notes = $3,
                deactivated_at = NULL
            "#
        )
        .bind(module_name)
        .bind(activated_by)
        .bind(notes)
        .execute(&self.pool)
        .await?;

        // Invalidate cache
        self.cache.remove(module_name);
        Ok(())
    }

    /// Disable a module
    pub async fn disable_module(
        &mut self,
        module_name: &str,
        _deactivated_by: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE module_activations
            SET enabled = false, deactivated_at = CURRENT_TIMESTAMP
            WHERE module_id = (SELECT id FROM modules WHERE name = $1)
            "#
        )
        .bind(module_name)
        .execute(&self.pool)
        .await?;

        // Invalidate cache
        self.cache.remove(module_name);
        Ok(())
    }

    /// Get enabled modules by category
    #[allow(dead_code)]
    pub async fn get_enabled_modules_by_category(
        &self,
        category: &str,
    ) -> Result<Vec<ModuleDb>, sqlx::Error> {
        let modules: Vec<ModuleDb> = sqlx::query_as(
            r#"
            SELECT m.id, m.name, m.display_name, m.description, m.category,
                   m.enabled_by_default, m.requires_license, m.created_at, m.updated_at
            FROM modules m
            JOIN module_activations ma ON m.id = ma.module_id
            WHERE m.category = $1 AND ma.enabled = true
            ORDER BY m.name
            "#
        )
        .bind(category)
        .fetch_all(&self.pool)
        .await?;

        Ok(modules)
    }

    /// Clear cache (useful after module changes)
    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

