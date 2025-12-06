# RBAC Integration Guide

## ✅ Implementované

### Helper funkcia
- `authenticate_and_authorize()` - centralizovaná autentifikácia a autorizácia

### Compliance Endpoints ✅
- `POST /api/v1/log_action` - `compliance.write`
- `GET /api/v1/logs` - `compliance.read`
- `POST /api/v1/shred_data` - `compliance.delete`
- `GET /api/v1/download_report` - `compliance.read`
- `POST /api/v1/revoke_access` - `admin` role only

## 📋 Zostáva implementovať

### Data Subject Rights Endpoints
- `GET /api/v1/data_subject/{user_id}/access` - `data_subject.read`
- `GET /api/v1/data_subject/{user_id}/export` - `data_subject.export`
- `PUT /api/v1/data_subject/{user_id}/rectify` - `data_subject.rectify`

### Human Oversight Endpoints
- `POST /api/v1/action/{seal_id}/require_approval` - `oversight.write`
- `POST /api/v1/action/{seal_id}/approve` - `oversight.approve`
- `POST /api/v1/action/{seal_id}/reject` - `oversight.approve`

### Risk Assessment Endpoints
- `GET /api/v1/risk_assessment/{seal_id}` - `risk.read`
- `GET /api/v1/risks` - `risk.read`

### Data Breach Endpoints
- `POST /api/v1/breach_report` - `breach.write`
- `GET /api/v1/breaches` - `breach.read`

### Consent Management Endpoints
- `POST /api/v1/consent` - `consent.write`
- `POST /api/v1/consent/withdraw` - `consent.write`
- `GET /api/v1/consent/{user_id}` - `consent.read`

### DPIA Endpoints
- `POST /api/v1/dpia` - `dpia.write`
- `PUT /api/v1/dpia/{dpia_id}` - `dpia.write`
- `GET /api/v1/dpias` - `dpia.read`

### Retention Policy Endpoints
- `POST /api/v1/retention/policy` - `retention.write`
- `POST /api/v1/retention/assign` - `retention.write`
- `GET /api/v1/retention/status/{record_type}/{record_id}` - `retention.read`
- `GET /api/v1/retention/policies` - `retention.read`
- `POST /api/v1/retention/execute_deletions` - `retention.write` (admin only)

### Monitoring Endpoints
- `POST /api/v1/monitoring/event` - `monitoring.write`
- `PUT /api/v1/monitoring/event/{event_id}` - `monitoring.write`
- `GET /api/v1/monitoring/events` - `monitoring.read`
- `GET /api/v1/monitoring/health/{system_id}` - `monitoring.read`

### Webhook Endpoints
- `POST /api/v1/webhooks` - `webhook.write`
- `GET /api/v1/webhooks` - `webhook.read`
- `PUT /api/v1/webhooks/{id}` - `webhook.write`
- `DELETE /api/v1/webhooks/{id}` - `webhook.delete`
- `GET /api/v1/webhooks/{id}/deliveries` - `webhook.read`

## 📝 Implementačný pattern

Pre každý endpoint:

```rust
pub async fn endpoint_name(
    // ... existing parameters ...
    http_req: HttpRequest,  // ← Pridať HttpRequest
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "resource", "action").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // ... existing endpoint logic ...
}
```

## 🔐 Permission Mapping

| Resource | Action | Endpoint Pattern |
|----------|--------|------------------|
| `compliance` | `read` | GET /logs, GET /download_report |
| `compliance` | `write` | POST /log_action |
| `compliance` | `delete` | POST /shred_data |
| `data_subject` | `read` | GET /data_subject/{id}/access |
| `data_subject` | `export` | GET /data_subject/{id}/export |
| `data_subject` | `rectify` | PUT /data_subject/{id}/rectify |
| `oversight` | `read` | GET /action/{id} |
| `oversight` | `approve` | POST /action/{id}/approve, POST /action/{id}/reject |
| `risk` | `read` | GET /risk_assessment/{id}, GET /risks |
| `breach` | `read` | GET /breaches |
| `breach` | `write` | POST /breach_report |
| `consent` | `read` | GET /consent/{id} |
| `consent` | `write` | POST /consent, POST /consent/withdraw |
| `dpia` | `read` | GET /dpias |
| `dpia` | `write` | POST /dpia, PUT /dpia/{id} |
| `retention` | `read` | GET /retention/* |
| `retention` | `write` | POST /retention/* |
| `monitoring` | `read` | GET /monitoring/* |
| `monitoring` | `write` | POST /monitoring/*, PUT /monitoring/* |
| `webhook` | `read` | GET /webhooks/* |
| `webhook` | `write` | POST /webhooks, PUT /webhooks/{id} |
| `webhook` | `delete` | DELETE /webhooks/{id} |

