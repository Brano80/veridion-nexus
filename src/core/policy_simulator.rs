// Policy Simulator - Impact Analysis for Operational Safety
// Simulates policy changes before enforcement to prevent production breakage

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Policy type for simulation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyType {
    SovereignLock,
    AgentRevocation,
    ConsentRequirement,
    ProcessingRestriction,
}

/// Simulation request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimulationRequest {
    pub policy_type: PolicyType,
    pub policy_config: serde_json::Value, // Policy-specific configuration
    pub time_range_days: Option<i64>, // How many days back to analyze (default: 7)
    pub agent_filter: Option<Vec<String>>, // Optional: simulate only for specific agents
    pub business_function_filter: Option<Vec<String>>, // Optional: filter by business function
    pub location_filter: Option<Vec<String>>, // Optional: filter by location/country
    pub time_offset_days: Option<i64>, // Optional: simulate "what if" scenario from N days ago
}

/// Simulation result showing impact
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimulationResult {
    pub policy_type: PolicyType,
    pub total_requests: i64,
    pub would_block: i64,
    pub would_allow: i64,
    pub affected_agents: Vec<AgentImpact>,
    pub affected_endpoints: HashMap<String, i64>, // endpoint -> count
    pub requests_by_country: HashMap<String, i64>, // country -> count
    pub requests_by_business_function: Option<HashMap<String, i32>>,
    pub requests_by_location: Option<HashMap<String, i32>>,
    pub estimated_impact: ImpactLevel,
    pub critical_agents: Vec<String>, // Agents that would be 100% blocked
    pub partial_impact_agents: Vec<String>, // Agents with mixed traffic
    pub simulation_timestamp: DateTime<Utc>,
}

/// Impact level assessment
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImpactLevel {
    Low,      // < 5% of traffic affected
    Medium,   // 5-20% of traffic affected
    High,     // 20-50% of traffic affected
    Critical, // > 50% of traffic affected
}

/// Agent-specific impact analysis
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentImpact {
    pub agent_id: String,
    pub total_requests: i64,
    pub would_block: i64,
    pub would_allow: i64,
    pub block_percentage: f64,
    pub affected_endpoints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_function: Option<String>, // Business function this agent belongs to
}

/// Policy Simulator Service
pub struct PolicySimulator;

impl PolicySimulator {
    /// Simulate a policy change and return impact analysis
    pub async fn simulate(
        db_pool: &PgPool,
        request: SimulationRequest,
    ) -> Result<SimulationResult, String> {
        let time_range = request.time_range_days.unwrap_or(7);
        let time_offset = request.time_offset_days.unwrap_or(0);
        let end_time = Utc::now() - Duration::days(time_offset);
        let start_time = end_time - Duration::days(time_range);

        // Get all compliance records in the time range (simplified - filters applied post-query for now)
        let records: Vec<ComplianceRecordForSimulation> = sqlx::query_as::<_, ComplianceRecordForSimulation>(
            "SELECT agent_id, action_summary, status, payload_hash, timestamp 
             FROM compliance_records 
             WHERE timestamp >= $1 AND timestamp <= $2
             ORDER BY timestamp DESC"
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_all(db_pool)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

        // Apply filters in memory (for simplicity - can be optimized with SQL filters later)
        let records: Vec<ComplianceRecordForSimulation> = records.into_iter()
            .filter(|r| {
                // Agent filter
                if let Some(agent_ids) = &request.agent_filter {
                    if !agent_ids.contains(&r.agent_id) {
                        return false;
                    }
                }
                // Location filter
                if let Some(loc_filters) = &request.location_filter {
                    let region = r.payload_hash.strip_prefix("region:")
                        .map(|s| s.to_uppercase())
                        .unwrap_or_else(|| "UNKNOWN".to_string());
                    if !loc_filters.iter().any(|loc| region.contains(&loc.to_uppercase())) {
                        return false;
                    }
                }
                true
            })
            .collect();

        let total_requests = records.len() as i64;

        // Apply policy simulation based on type
        match request.policy_type {
            PolicyType::SovereignLock => {
                Self::simulate_sovereign_lock(records, &request.policy_config)
            }
            PolicyType::AgentRevocation => {
                Self::simulate_agent_revocation(records, &request.policy_config)
            }
            PolicyType::ConsentRequirement => {
                Self::simulate_consent_requirement(records, &request.policy_config)
            }
            PolicyType::ProcessingRestriction => {
                Self::simulate_processing_restriction(records, &request.policy_config)
            }
        }
        .map(|mut result| {
            result.total_requests = total_requests;
            result.policy_type = request.policy_type.clone();
            result.simulation_timestamp = Utc::now();
            result.estimated_impact = Self::calculate_impact_level(
                result.would_block,
                total_requests,
            );
            
            // Enrich agent impacts with business function data (async lookup)
            // Note: This is done synchronously here - in production, consider batching
            for agent_impact in &mut result.affected_agents {
                if agent_impact.business_function.is_none() {
                    // Will be populated by async lookup if available
                    agent_impact.business_function = None;
                }
            }
            
            result
        })
    }

    /// Simulate Sovereign Lock policy
    fn simulate_sovereign_lock(
        records: Vec<ComplianceRecordForSimulation>,
        config: &serde_json::Value,
    ) -> Result<SimulationResult, String> {
        // Extract blocked countries from config
        let blocked_countries: Vec<String> = config
            .get("blocked_countries")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_uppercase()))
                    .collect()
            })
            .unwrap_or_else(|| vec!["US".to_string(), "CN".to_string(), "RU".to_string()]);

        let mut would_block = 0;
        let mut would_allow = 0;
        let mut agent_stats: HashMap<String, (i64, i64)> = HashMap::new(); // (blocked, total)
        let mut endpoint_stats: HashMap<String, i64> = HashMap::new();
        let mut country_stats: HashMap<String, i64> = HashMap::new();

        for record in &records {
            // Extract region from payload_hash (format: "region:US" or "region:DE")
            let region = record
                .payload_hash
                .strip_prefix("region:")
                .map(|s| s.to_uppercase())
                .unwrap_or_else(|| "UNKNOWN".to_string());

            // Extract endpoint from action_summary
            let endpoint = Self::extract_endpoint(&record.action_summary);

            // Check if region would be blocked
            let is_blocked = blocked_countries.contains(&region) || region == "UNKNOWN";

            if is_blocked {
                would_block += 1;
                let stats = agent_stats.entry(record.agent_id.clone()).or_insert((0, 0));
                stats.0 += 1;
            } else {
                would_allow += 1;
            }
            let stats = agent_stats.entry(record.agent_id.clone()).or_insert((0, 0));
            stats.1 += 1;
            
            *endpoint_stats.entry(endpoint).or_insert(0) += 1;
            *country_stats.entry(region).or_insert(0) += 1;
        }

        // Build agent impact list
        let mut agent_impacts: Vec<AgentImpact> = agent_stats
            .iter()
            .map(|(agent_id, (blocked, total))| {
                let block_percentage = if *total > 0 {
                    (*blocked as f64 / *total as f64) * 100.0
                } else {
                    0.0
                };

                // Get affected endpoints for this agent
                let affected_endpoints: Vec<String> = records
                    .iter()
                    .filter(|r| r.agent_id == *agent_id)
                    .map(|r| Self::extract_endpoint(&r.action_summary))
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                // Get business function for this agent (async, but we'll do it synchronously here)
                // Note: This is a simplified approach - in production, you might want to batch these lookups
                let business_function = None; // Will be populated in the calling function if needed
                
                AgentImpact {
                    agent_id: agent_id.clone(),
                    total_requests: *total,
                    would_block: *blocked,
                    would_allow: *total - *blocked,
                    block_percentage,
                    affected_endpoints,
                    business_function,
                }
            })
            .collect();

        // Sort by block percentage (highest first)
        agent_impacts.sort_by(|a, b| {
            b.block_percentage.partial_cmp(&a.block_percentage).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Identify critical and partial impact agents
        let critical_agents: Vec<String> = agent_impacts
            .iter()
            .filter(|a| a.block_percentage == 100.0 && a.total_requests > 0)
            .map(|a| a.agent_id.clone())
            .collect();

        let partial_impact_agents: Vec<String> = agent_impacts
            .iter()
            .filter(|a| a.block_percentage > 0.0 && a.block_percentage < 100.0)
            .map(|a| a.agent_id.clone())
            .collect();

        Ok(SimulationResult {
            policy_type: PolicyType::SovereignLock,
            total_requests: records.len() as i64,
            would_block,
            would_allow,
            affected_agents: agent_impacts,
            affected_endpoints: endpoint_stats,
            requests_by_country: country_stats,
            requests_by_business_function: None,
            requests_by_location: None,
            estimated_impact: ImpactLevel::Low, // Will be calculated later
            critical_agents,
            partial_impact_agents,
            simulation_timestamp: Utc::now(),
        })
    }

    /// Simulate Agent Revocation policy
    fn simulate_agent_revocation(
        records: Vec<ComplianceRecordForSimulation>,
        config: &serde_json::Value,
    ) -> Result<SimulationResult, String> {
        let revoked_agents: Vec<String> = config
            .get("revoked_agents")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .ok_or_else(|| "Missing 'revoked_agents' in config".to_string())?;

        let mut would_block = 0;
        let mut would_allow = 0;
        let mut agent_stats: HashMap<String, (i64, i64)> = HashMap::new();
        let mut endpoint_stats: HashMap<String, i64> = HashMap::new();
        let mut country_stats: HashMap<String, i64> = HashMap::new();

        for record in &records {
            let is_blocked = revoked_agents.contains(&record.agent_id);
            let endpoint = Self::extract_endpoint(&record.action_summary);
            let region = record
                .payload_hash
                .strip_prefix("region:")
                .map(|s| s.to_uppercase())
                .unwrap_or_else(|| "UNKNOWN".to_string());

            if is_blocked {
                would_block += 1;
                let stats = agent_stats.entry(record.agent_id.clone()).or_insert((0, 0));
                stats.0 += 1;
            } else {
                would_allow += 1;
            }
            let stats = agent_stats.entry(record.agent_id.clone()).or_insert((0, 0));
            stats.1 += 1;
            *endpoint_stats.entry(endpoint).or_insert(0) += 1;
            *country_stats.entry(region).or_insert(0) += 1;
        }

        let agent_impacts: Vec<AgentImpact> = agent_stats
            .iter()
            .map(|(agent_id, (blocked, total))| {
                let block_percentage = if *total > 0 {
                    (*blocked as f64 / *total as f64) * 100.0
                } else {
                    0.0
                };

                let affected_endpoints: Vec<String> = records
                    .iter()
                    .filter(|r| r.agent_id == *agent_id)
                    .map(|r| Self::extract_endpoint(&r.action_summary))
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                // Get business function for this agent (async, but we'll do it synchronously here)
                // Note: This is a simplified approach - in production, you might want to batch these lookups
                let business_function = None; // Will be populated in the calling function if needed
                
                AgentImpact {
                    agent_id: agent_id.clone(),
                    total_requests: *total,
                    would_block: *blocked,
                    would_allow: *total - *blocked,
                    block_percentage,
                    affected_endpoints,
                    business_function,
                }
            })
            .collect();

        let critical_agents = revoked_agents;

        Ok(SimulationResult {
            policy_type: PolicyType::AgentRevocation,
            total_requests: records.len() as i64,
            would_block,
            would_allow,
            affected_agents: agent_impacts,
            affected_endpoints: endpoint_stats,
            requests_by_country: country_stats,
            requests_by_business_function: None,
            requests_by_location: None,
            estimated_impact: ImpactLevel::Low,
            critical_agents,
            partial_impact_agents: vec![],
            simulation_timestamp: Utc::now(),
        })
    }

    /// Simulate Consent Requirement policy
    fn simulate_consent_requirement(
        records: Vec<ComplianceRecordForSimulation>,
        _config: &serde_json::Value,
    ) -> Result<SimulationResult, String> {
        // For now, return a basic simulation
        // In a full implementation, this would check user consent status
        Ok(SimulationResult {
            policy_type: PolicyType::ConsentRequirement,
            total_requests: records.len() as i64,
            would_block: 0,
            would_allow: records.len() as i64,
            affected_agents: vec![],
            affected_endpoints: HashMap::new(),
            requests_by_country: HashMap::new(),
            requests_by_business_function: None,
            requests_by_location: None,
            estimated_impact: ImpactLevel::Low,
            critical_agents: vec![],
            partial_impact_agents: vec![],
            simulation_timestamp: Utc::now(),
        })
    }

    /// Simulate Processing Restriction policy
    fn simulate_processing_restriction(
        records: Vec<ComplianceRecordForSimulation>,
        _config: &serde_json::Value,
    ) -> Result<SimulationResult, String> {
        // For now, return a basic simulation
        Ok(SimulationResult {
            policy_type: PolicyType::ProcessingRestriction,
            total_requests: records.len() as i64,
            would_block: 0,
            would_allow: records.len() as i64,
            affected_agents: vec![],
            affected_endpoints: HashMap::new(),
            requests_by_country: HashMap::new(),
            requests_by_business_function: None,
            requests_by_location: None,
            estimated_impact: ImpactLevel::Low,
            critical_agents: vec![],
            partial_impact_agents: vec![],
            simulation_timestamp: Utc::now(),
        })
    }

    /// Calculate impact level based on block percentage
    fn calculate_impact_level(blocked: i64, total: i64) -> ImpactLevel {
        if total == 0 {
            return ImpactLevel::Low;
        }

        let percentage = (blocked as f64 / total as f64) * 100.0;

        if percentage >= 50.0 {
            ImpactLevel::Critical
        } else if percentage >= 20.0 {
            ImpactLevel::High
        } else if percentage >= 5.0 {
            ImpactLevel::Medium
        } else {
            ImpactLevel::Low
        }
    }

    /// Extract endpoint from action summary
    fn extract_endpoint(action_summary: &str) -> String {
        // Try to extract URL/endpoint from action summary
        // Format might be: "PROXY_BLOCKED: Attempted connection to https://api.openai.com/v1/chat/completions"
        if let Some(url_start) = action_summary.find("http") {
            let url_part = &action_summary[url_start..];
            // Find end of URL (space, newline, or end of string)
            let url_end = url_part
                .find(' ')
                .or_else(|| url_part.find('\n'))
                .unwrap_or(url_part.len());
            let url = &url_part[..url_end];
            
            // Extract hostname manually (simpler than parsing full URL)
            // Look for "://" and then find next "/" or end
            if let Some(proto_end) = url.find("://") {
                let after_proto = &url[proto_end + 3..];
                if let Some(host_end) = after_proto.find('/') {
                    return after_proto[..host_end].to_string();
                } else {
                    return after_proto.to_string();
                }
            }
        }
        "unknown".to_string()
    }
}

/// Internal struct for simulation queries
#[derive(sqlx::FromRow)]
struct ComplianceRecordForSimulation {
    agent_id: String,
    action_summary: String,
    status: String,
    payload_hash: String,
    timestamp: DateTime<Utc>,
}

