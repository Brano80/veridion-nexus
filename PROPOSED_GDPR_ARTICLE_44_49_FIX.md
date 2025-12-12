# Návrh úpravy: GDPR Article 44-49 validácia pred blokovaním non-EU requestov

## 🔍 Aktuálny stav

### Problém
V `/log_action` endpointe sa non-EU requesty blokujú **bez kontroly GDPR Article 44-49**:
- Riadok 368-380: Hardcoded check pre US/CN/RU krajiny
- **Chýba**: Kontrola, či existuje schválený transfer s DPA/SCCs
- **Chýba**: Získanie `company_id` z requestu

### Čo už existuje
✅ `GDPRArticle4449Module` s metódami:
- `check_transfer_allowed_for_proxy()` - kontroluje schválené transfery
- `has_approved_transfer()` - kontroluje, či existuje approved transfer pre krajinu
- `validate_transfer()` - validuje transfer s legal_basis

✅ V proxy endpointe (riadok 6348-6716) už existuje GDPR check s company_id

---

## 📋 Navrhovaná úprava

### 1. Pridať získanie `company_id` v `/log_action` endpointe

**Kde:** Pred SOVEREIGN LOCK check (pred riadok 368)

```rust
// Helper: Get company_id from request (X-Company-ID header or from user_id)
let company_id: Option<uuid::Uuid> = {
    // Try X-Company-ID header first
    if let Some(company_id_str) = http_req.headers()
        .get("X-Company-ID")
        .and_then(|h| h.to_str().ok())
    {
        uuid::Uuid::parse_str(company_id_str).ok()
    } else if let Some(ref user_id) = req.user_id {
        // Try to get company_id from user_id (if users table has company_id)
        sqlx::query_scalar::<_, Option<uuid::Uuid>>(
            "SELECT company_id FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&data.db_pool)
        .await
        .ok()
        .flatten()
    } else {
        None
    }
};
```

### 2. Upraviť SOVEREIGN LOCK check s GDPR Article 44-49 validáciou

**Kde:** Namiesto riadkov 368-380

```rust
// B. SOVEREIGN LOCK with GDPR Article 44-49 validation
// Check for blocked regions (case-insensitive, also catches AWS region patterns like "us-east-1")
let target = req.target_region.as_deref().unwrap_or("").to_uppercase();
let is_non_eu = target == "US" 
    || target == "CN"
    || target == "RU"
    || target == "USA"
    || target.starts_with("US-")  // Catches "US-EAST-1", "US-WEST-2", etc.
    || target.contains("UNITED STATES")
    || target == "CHINA"
    || target == "RUSSIA"
    || !crate::core::sovereign_lock::EU_EEA_WHITELIST.contains(&target.as_str());

// If non-EU detected, check GDPR Article 44-49 before blocking
let is_violation = if is_non_eu {
    // Check if there's an approved transfer with DPA/SCCs
    if let Some(cid) = company_id {
        use crate::modules::gdpr::article_44_49_international_transfers::GDPRArticle4449Module;
        let gdpr_module = GDPRArticle4449Module::new(data.db_pool.clone());
        
        match gdpr_module.check_transfer_allowed_for_proxy(cid, &target).await {
            Ok((allowed, reason, requires_alert)) => {
                if allowed {
                    // Transfer is allowed (DPA/SCCs in place) - log and allow
                    log::info!("SOVEREIGN_LOCK_ALLOWED: {} -> {} ({})", req.agent_id, target, reason);
                    false // Not a violation - transfer is allowed
                } else {
                    // Transfer not allowed - block with compliance alert
                    log::warn!("SOVEREIGN_LOCK_BLOCKED: {} -> {} ({})", req.agent_id, target, reason);
                    true // Violation - block
                }
            }
            Err(e) => {
                // Error checking GDPR module - log and block conservatively
                log::error!("Error checking GDPR Article 44-49 for {}: {}", target, e);
                true // Block on error
            }
        }
    } else {
        // No company_id - use default blocking behavior
        log::warn!("SOVEREIGN_LOCK_BLOCKED: {} -> {} (No company_id provided)", req.agent_id, target);
        true // Block if no company_id
    }
} else {
    false // EU/EEA country - no violation
};

let status = if is_violation { "BLOCKED (SOVEREIGNTY)" } else { "COMPLIANT" };
```

### 3. Upraviť shadow mode logging

**Kde:** Riadky 383-454 - pridať GDPR check aj do shadow mode

```rust
// In shadow mode, log what would happen but don't block
if is_shadow_mode && is_violation {
    // Log to shadow_mode_logs with GDPR check info
    let record_id = uuid::Uuid::new_v4();
    let log_hash = crate::core::privacy_bridge::hash_payload(&req.payload);
    
    // Check if transfer would be allowed with GDPR check
    let would_be_allowed = if let Some(cid) = company_id {
        use crate::modules::gdpr::article_44_49_international_transfers::GDPRArticle4449Module;
        let gdpr_module = GDPRArticle4449Module::new(data.db_pool.clone());
        gdpr_module.check_transfer_allowed_for_proxy(cid, &target).await
            .map(|(allowed, _, _)| allowed)
            .unwrap_or(false)
    } else {
        false
    };
    
    let _ = sqlx::query(
        "INSERT INTO shadow_mode_logs (
            id, agent_id, action_summary, action_type, payload_hash,
            target_region, would_block, would_allow, policy_applied,
            risk_level, detected_country, timestamp
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
    )
    .bind(record_id)
    .bind(&req.agent_id)
    .bind(&format!("{}: {}", req.agent_id, req.action))
    .bind(&req.action)
    .bind(&log_hash)
    .bind(&target)
    .bind(!would_be_allowed)  // would_block = true if GDPR check fails
    .bind(would_be_allowed)   // would_allow = true if GDPR check passes
    .bind("SOVEREIGN_LOCK")
    .bind("HIGH")
    .bind(&target)
    .bind(chrono::Utc::now())
    .execute(&data.db_pool)
    .await;
    
    if would_be_allowed {
        log::info!("SHADOW MODE: Would allow {} -> {} (GDPR Article 44-49: DPA/SCCs in place)", req.agent_id, target);
    } else {
        log::warn!("SHADOW MODE: Would block {} -> {} (Country: {}, No DPA/SCCs)", req.agent_id, req.action, target);
    }
    
    // Continue processing - don't block in shadow mode
} else if is_shadow_mode && !is_violation {
    // ... existing code for allowed actions ...
}
```

### 4. Upraviť blocking response s compliance alertom

**Kde:** Namiesto jednoduchého blocking response (ak existuje)

```rust
// If violation and not in shadow mode, block the request
if !is_shadow_mode && is_violation {
    // Check if this requires compliance alert (no DPA/SCCs)
    let requires_alert = if let Some(cid) = company_id {
        use crate::modules::gdpr::article_44_49_international_transfers::GDPRArticle4449Module;
        let gdpr_module = GDPRArticle4449Module::new(data.db_pool.clone());
        gdpr_module.check_transfer_allowed_for_proxy(cid, &target).await
            .map(|(_, _, alert)| alert)
            .unwrap_or(true)
    } else {
        true
    };
    
    return HttpResponse::Forbidden().json(serde_json::json!({
        "error": if requires_alert { "COMPLIANCE_ALERT" } else { "SOVEREIGN_LOCK_VIOLATION" },
        "message": format!("Data sovereignty violation: target region {} is not in EU/EEA. Transfer to non-EU country requires approved data transfer agreement (DPA/SCCs).", target),
        "target_region": target,
        "status": "BLOCKED",
        "compliance_alert": requires_alert,
        "action_required": if requires_alert {
            "Please register a data transfer with DPA/SCCs in the Wizard or Settings to allow transfers to this country."
        } else {
            "Transfer to non-EU country is blocked by policy."
        }
    }));
}
```

---

## 📝 Zhrnutie zmien

### Súbory na úpravu:
1. **`src/routes.rs`** - `/log_action` endpoint
   - Pridať získanie `company_id` (pred riadok 368)
   - Upraviť SOVEREIGN LOCK check s GDPR validáciou (riadky 368-380)
   - Upraviť shadow mode logging (riadky 383-454)
   - Upraviť blocking response (ak existuje)

### Výhody:
✅ Non-EU requesty sa neblokujú, ak existuje schválený transfer s DPA/SCCs
✅ Compliance alert sa zobrazí len ak skutočne chýba DPA/SCCs
✅ Shadow mode správne loguje, či by request bol povolený s GDPR checkom
✅ Konzistentné správanie s proxy endpointom

### Testovanie:
1. Test s `company_id` a schváleným transferom → má povoliť
2. Test s `company_id` bez schváleného transferu → má blokovať s alertom
3. Test bez `company_id` → má blokovať (konzervatívne)
4. Test v shadow mode → má logovať správne

---

## ⚠️ Poznámky

- **Company ID získanie**: Môže byť z `X-Company-ID` headeru alebo z `user_id` (ak users table má company_id)
- **Fallback**: Ak nie je company_id, blokovať konzervatívne (bezpečnejšie)
- **Performance**: GDPR check je async DB query - môže pridať malé oneskorenie, ale je nevyhnutné pre compliance

