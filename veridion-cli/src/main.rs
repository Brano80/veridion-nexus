use clap::{Parser, Subcommand};
use colored::*;
use reqwest::Client;
use serde_json::json;
use std::env;

const DEFAULT_API_URL: &str = "http://127.0.0.1:8080/api/v1";

#[derive(Parser)]
#[command(name = "veridion")]
#[command(about = "Veridion Nexus CLI - Test, simulate, and manage policies", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(long, default_value = DEFAULT_API_URL)]
    api_url: String,
    
    #[arg(long, env = "VERIDION_API_KEY")]
    api_key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Test a policy configuration
    Test {
        #[arg(short, long)]
        policy_type: String,
        #[arg(short, long)]
        config: String,
    },
    /// Simulate policy impact
    Simulate {
        #[arg(short, long)]
        policy_type: String,
        #[arg(short, long)]
        config: String,
        #[arg(short, long, default_value = "7")]
        days: u32,
    },
    /// Rollback a policy to a previous version
    Rollback {
        #[arg(short, long)]
        policy_id: String,
        #[arg(short, long)]
        version: Option<u32>,
        #[arg(short, long)]
        dry_run: bool,
        #[arg(short, long)]
        notes: Option<String>,
    },
    /// Get policy health status
    Health {
        #[arg(short, long)]
        policy_id: String,
    },
    /// Get shadow mode analytics
    Shadow {
        #[arg(short, long, default_value = "7")]
        days: u32,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let api_key = cli.api_key.or_else(|| env::var("VERIDION_API_KEY").ok())
        .ok_or_else(|| anyhow::anyhow!("API key required. Set VERIDION_API_KEY environment variable or use --api-key"))?;
    
    let client = Client::new();
    let base_url = cli.api_url.trim_end_matches('/');
    
    match cli.command {
        Commands::Test { policy_type, config } => {
            println!("{}", "Testing policy...".cyan());
            let config_json: serde_json::Value = serde_json::from_str(&config)?;
            
            let response = client
                .post(&format!("{}/policies/preview-impact", base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .query(&[("policy_type", &policy_type), ("time_range_days", "7")])
                .json(&json!({
                    "policy_config": config_json
                }))
                .send()
                .await?;
            
            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                println!("{}", "✓ Policy test completed".green());
                println!("\n{}", "Simulation Results:".bold());
                if let Some(sim) = result.get("simulation_result") {
                    println!("  Total Requests: {}", sim.get("total_requests").unwrap_or(&json!(0)));
                    println!("  Would Block: {}", sim.get("would_block").unwrap_or(&json!(0)));
                    println!("  Would Allow: {}", sim.get("would_allow").unwrap_or(&json!(0)));
                }
                if let Some(impact) = result.get("business_impact") {
                    println!("\n{}", "Business Impact:".bold());
                    println!("  {}", impact.get("recommendation").unwrap_or(&json!("N/A")));
                }
                if let Some(cost) = result.get("cost_impact") {
                    println!("\n{}", "Cost Impact:".bold());
                    println!("  Estimated Total Cost: ${:.2}", 
                        cost.get("estimated_total_cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0));
                }
            } else {
                let error: serde_json::Value = response.json().await?;
                eprintln!("{}", "✗ Policy test failed".red());
                eprintln!("  Error: {}", error.get("message").unwrap_or(&json!("Unknown error")));
            }
        }
        Commands::Simulate { policy_type, config, days } => {
            println!("{}", format!("Simulating policy impact over {} days...", days).cyan());
            let config_json: serde_json::Value = serde_json::from_str(&config)?;
            
            let response = client
                .post(&format!("{}/policies/simulate", base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&json!({
                    "policy_type": policy_type,
                    "policy_config": config_json,
                    "time_range_days": days
                }))
                .send()
                .await?;
            
            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                println!("{}", "✓ Simulation completed".green());
                println!("\n{}", "Results:".bold());
                println!("  Total Requests: {}", result.get("total_requests").unwrap_or(&json!(0)));
                println!("  Would Block: {}", result.get("would_block").unwrap_or(&json!(0)));
                println!("  Would Allow: {}", result.get("would_allow").unwrap_or(&json!(0)));
                if let Some(impact) = result.get("estimated_impact") {
                    println!("  Estimated Impact: {}", impact);
                }
            } else {
                let error: serde_json::Value = response.json().await?;
                eprintln!("{}", "✗ Simulation failed".red());
                eprintln!("  Error: {}", error.get("message").unwrap_or(&json!("Unknown error")));
            }
        }
        Commands::Rollback { policy_id, version, dry_run, notes } => {
            if dry_run {
                println!("{}", "Testing rollback (dry-run)...".cyan());
            } else {
                println!("{}", "Rolling back policy...".cyan());
            }
            
            let response = client
                .post(&format!("{}/policies/{}/rollback", base_url, policy_id))
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&json!({
                    "target_version": version,
                    "dry_run": dry_run,
                    "notes": notes
                }))
                .send()
                .await?;
            
            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                if dry_run {
                    println!("{}", "✓ Rollback test completed".green());
                    println!("\n{}", "Simulation Results:".bold());
                } else {
                    println!("{}", "✓ Policy rolled back successfully".green());
                    if let Some(duration) = result.get("rollback_duration_sec") {
                        println!("  Rollback Duration: {:.2}s", duration.as_f64().unwrap_or(0.0));
                    }
                    if let Some(sla_met) = result.get("sla_met") {
                        if sla_met.as_bool().unwrap_or(false) {
                            println!("  {} SLA Met (< 30s)", "✓".green());
                        } else {
                            println!("  {} SLA Not Met (> 30s)", "✗".red());
                        }
                    }
                }
                println!("  Message: {}", result.get("message").unwrap_or(&json!("N/A")));
            } else {
                let error: serde_json::Value = response.json().await?;
                eprintln!("{}", "✗ Rollback failed".red());
                eprintln!("  Error: {}", error.get("message").unwrap_or(&json!("Unknown error")));
            }
        }
        Commands::Health { policy_id } => {
            println!("{}", "Fetching policy health...".cyan());
            
            let response = client
                .get(&format!("{}/policies/{}/health", base_url, policy_id))
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await?;
            
            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                println!("{}", "✓ Policy health retrieved".green());
                println!("\n{}", "Health Status:".bold());
                if let Some(status) = result.get("status") {
                    println!("  Status: {}", status);
                }
                if let Some(success_rate) = result.get("success_rate") {
                    println!("  Success Rate: {:.2}%", success_rate.as_f64().unwrap_or(0.0));
                }
                if let Some(error_rate) = result.get("error_rate") {
                    println!("  Error Rate: {:.2}%", error_rate.as_f64().unwrap_or(0.0));
                }
            } else {
                let error: serde_json::Value = response.json().await?;
                eprintln!("{}", "✗ Failed to fetch policy health".red());
                eprintln!("  Error: {}", error.get("message").unwrap_or(&json!("Unknown error")));
            }
        }
        Commands::Shadow { days } => {
            println!("{}", format!("Fetching shadow mode analytics (last {} days)...", days).cyan());
            
            let response = client
                .get(&format!("{}/analytics/shadow-mode?days={}", base_url, days))
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await?;
            
            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                println!("{}", "✓ Shadow mode analytics retrieved".green());
                println!("\n{}", "Analytics:".bold());
                if let Some(total) = result.get("total_logs") {
                    println!("  Total Logs: {}", total);
                }
                if let Some(would_block) = result.get("would_block_count") {
                    println!("  Would Block: {}", would_block);
                }
                if let Some(would_allow) = result.get("would_allow_count") {
                    println!("  Would Allow: {}", would_allow);
                }
            } else {
                let error: serde_json::Value = response.json().await?;
                eprintln!("{}", "✗ Failed to fetch shadow mode analytics".red());
                eprintln!("  Error: {}", error.get("message").unwrap_or(&json!("Unknown error")));
            }
        }
    }
    
    Ok(())
}

