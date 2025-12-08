use printpdf::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufWriter;
use utoipa::ToSchema;

/// A compliance record for the Annex IV report
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ComplianceRecord {
    /// Timestamp when the action was logged
    #[schema(example = "2024-01-15 14:30:00")]
    pub timestamp: String,
    /// Summary of the agent action
    #[schema(example = "agent-001: Credit Check")]
    pub action_summary: String,
    /// Qualified Electronic Seal ID from eIDAS provider
    #[schema(example = "SEAL-2024-01-15-ABC123")]
    pub seal_id: String,
    /// Compliance status of the action
    #[schema(example = "COMPLIANT")]
    pub status: String,
    /// User notification status (EU AI Act Article 13)
    #[schema(example = true)]
    pub user_notified: Option<bool>,
    /// Timestamp when user was notified
    #[schema(example = "2024-01-15 14:30:05")]
    pub notification_timestamp: Option<String>,
    /// Human oversight status (EU AI Act Article 14)
    #[schema(example = "APPROVED")]
    pub human_oversight_status: Option<String>, // "PENDING", "APPROVED", "REJECTED"
    /// Risk assessment level
    #[schema(example = "MEDIUM")]
    pub risk_level: Option<String>, // "LOW", "MEDIUM", "HIGH"
    /// User ID for data subject rights (GDPR)
    #[schema(example = "user-123")]
    pub user_id: Option<String>,
    // Extended Annex IV fields
    /// AI system lifecycle stage (DEVELOPMENT, TRAINING, DEPLOYMENT, MONITORING, DECOMMISSIONING)
    #[schema(example = "DEPLOYMENT")]
    pub lifecycle_stage: Option<String>,
    /// Training data sources and characteristics
    #[schema(example = r#"["Public datasets", "Internal data"]"#)]
    pub training_data_sources: Option<Vec<String>>,
    /// Performance metrics and evaluation methods
    #[schema(example = r#"{"accuracy": 0.95, "precision": 0.92}"#)]
    pub performance_metrics: Option<serde_json::Value>,
    /// Post-market monitoring results
    #[schema(example = "No incidents detected")]
    pub post_market_monitoring: Option<String>,
    /// Human oversight procedures applied
    #[schema(example = "Automated review with human escalation")]
    pub human_oversight_procedures: Option<String>,
    /// Risk management measures implemented
    #[schema(example = r#"["Encryption", "Access controls", "Audit logging"]"#)]
    pub risk_management_measures: Option<Vec<String>>,
}

/// Generate an Annex IV compliance report PDF
/// 
/// # Arguments
/// 
/// * `records` - Vector of compliance records to include in the report
/// * `output_path` - Path where the PDF file should be saved
/// 
/// # Returns
/// 
/// * `Ok(())` if the PDF was generated successfully
/// * `Err(String)` if there was an error generating the PDF
pub fn generate_report(records: &Vec<ComplianceRecord>, output_path: &str) -> Result<(), String> {
    let (doc, page1, layer1) = PdfDocument::new("Veridion Annex IV Report", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Header Text
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("Failed to add font: {:?}", e))?;
    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None))); // Black Text
    current_layer.use_text("VERIDION NEXUS | COMPLIANCE REPORT", 18.0, Mm(10.0), Mm(280.0), &font);
    
    let font_reg = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Failed to add font: {:?}", e))?;
    current_layer.use_text("EU AI Act - Annex IV Technical Documentation", 10.0, Mm(10.0), Mm(270.0), &font_reg);

    // --- TABLE HEADERS ---
    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None))); // Black Text
    let y_start = 250.0;
    current_layer.use_text("TIMESTAMP", 9.0, Mm(10.0), Mm(y_start), &font);
    current_layer.use_text("AGENT ACTION", 9.0, Mm(50.0), Mm(y_start), &font);
    current_layer.use_text("QUALIFIED SEAL ID (eIDAS)", 9.0, Mm(110.0), Mm(y_start), &font);

    // --- DATA ROWS ---
    let mut y_pos = y_start - 10.0;
    for record in records {
        if y_pos < 20.0 {
            // Simple logic: Stop writing if page is full (Pagination skipped for MVP simplicity)
            break; 
        }

        current_layer.use_text(&record.timestamp, 8.0, Mm(10.0), Mm(y_pos), &font_reg);
        current_layer.use_text(&record.action_summary, 8.0, Mm(50.0), Mm(y_pos), &font_reg);
        
        // Shorten seal ID for display to fit
        let short_seal = if record.seal_id.len() > 30 {
            format!("{}...", &record.seal_id[0..30])
        } else {
            record.seal_id.clone()
        };
        current_layer.use_text(&short_seal, 8.0, Mm(110.0), Mm(y_pos), &font_reg);

        y_pos -= 10.0;
    }

    // --- FOOTER ---
    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    current_layer.use_text("Generated automatically by Veridion Nexus - Confidential", 8.0, Mm(10.0), Mm(10.0), &font_reg);

    doc.save(&mut BufWriter::new(File::create(output_path).map_err(|e| e.to_string())?)).map_err(|e| e.to_string())?;
    Ok(())
}

/// Export Annex IV report to JSON format
pub fn export_to_json(records: &Vec<ComplianceRecord>, output_path: &str) -> Result<(), String> {
    let json_content = serde_json::to_string_pretty(records)
        .map_err(|e| format!("Failed to serialize to JSON: {}", e))?;
    
    std::fs::write(output_path, json_content)
        .map_err(|e| format!("Failed to write JSON file: {}", e))?;
    
    Ok(())
}

/// Export Annex IV report to XML format
pub fn export_to_xml(records: &Vec<ComplianceRecord>, output_path: &str) -> Result<(), String> {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<annex_iv_report xmlns="https://veridion.nexus/annex-iv">
  <generated_at>"#);
    
    xml.push_str(&chrono::Utc::now().to_rfc3339());
    xml.push_str(r#"</generated_at>
  <records>
"#);
    
    for record in records {
        xml.push_str("    <record>\n");
        xml.push_str(&format!("      <timestamp>{}</timestamp>\n", escape_xml(&record.timestamp)));
        xml.push_str(&format!("      <action_summary>{}</action_summary>\n", escape_xml(&record.action_summary)));
        xml.push_str(&format!("      <seal_id>{}</seal_id>\n", escape_xml(&record.seal_id)));
        xml.push_str(&format!("      <status>{}</status>\n", escape_xml(&record.status)));
        
        if let Some(ref risk_level) = record.risk_level {
            xml.push_str(&format!("      <risk_level>{}</risk_level>\n", escape_xml(risk_level)));
        }
        
        if let Some(ref lifecycle_stage) = record.lifecycle_stage {
            xml.push_str(&format!("      <lifecycle_stage>{}</lifecycle_stage>\n", escape_xml(lifecycle_stage)));
        }
        
        if let Some(ref training_data) = record.training_data_sources {
            xml.push_str("      <training_data_sources>\n");
            for source in training_data {
                xml.push_str(&format!("        <source>{}</source>\n", escape_xml(source)));
            }
            xml.push_str("      </training_data_sources>\n");
        }
        
        if let Some(ref measures) = record.risk_management_measures {
            xml.push_str("      <risk_management_measures>\n");
            for measure in measures {
                xml.push_str(&format!("        <measure>{}</measure>\n", escape_xml(measure)));
            }
            xml.push_str("      </risk_management_measures>\n");
        }
        
        xml.push_str("    </record>\n");
    }
    
    xml.push_str("  </records>\n");
    xml.push_str("</annex_iv_report>\n");
    
    std::fs::write(output_path, xml)
        .map_err(|e| format!("Failed to write XML file: {}", e))?;
    
    Ok(())
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

/// Export format enumeration
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Pdf,
    Json,
    Xml,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pdf" => Some(ExportFormat::Pdf),
            "json" => Some(ExportFormat::Json),
            "xml" => Some(ExportFormat::Xml),
            _ => None,
        }
    }
    
    pub fn content_type(&self) -> &'static str {
        match self {
            ExportFormat::Pdf => "application/pdf",
            ExportFormat::Json => "application/json",
            ExportFormat::Xml => "application/xml",
        }
    }
    
    pub fn file_extension(&self) -> &'static str {
        match self {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Json => "json",
            ExportFormat::Xml => "xml",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_generation() {
        // Create a vector of 3 dummy ComplianceRecord items
        let records = vec![
            ComplianceRecord {
                timestamp: "2024-01-15 10:30:00".to_string(),
                action_summary: "Trade Executed".to_string(),
                seal_id: "QES_SEAL_abc123".to_string(),
                status: "COMPLETED".to_string(),
                user_notified: None,
                notification_timestamp: None,
                human_oversight_status: None,
                risk_level: None,
                user_id: None,
            },
            ComplianceRecord {
                timestamp: "2024-01-15 11:45:00".to_string(),
                action_summary: "Data Shredded".to_string(),
                seal_id: "QES_SEAL_def456".to_string(),
                status: "COMPLETED".to_string(),
                user_notified: None,
                notification_timestamp: None,
                human_oversight_status: None,
                risk_level: None,
                user_id: None,
            },
            ComplianceRecord {
                timestamp: "2024-01-15 12:00:00".to_string(),
                action_summary: "Compliance Check".to_string(),
                seal_id: "QES_SEAL_ghi789".to_string(),
                status: "VERIFIED".to_string(),
                user_notified: None,
                notification_timestamp: None,
                human_oversight_status: None,
                risk_level: None,
                user_id: None,
            },
        ];
        
        // Call generate_report
        let result = generate_report(&records, "test_report.pdf");
        assert!(result.is_ok(), "PDF generation should succeed");
        
        // Assert that the file test_report.pdf exists
        assert!(std::path::Path::new("test_report.pdf").exists(), "PDF file should exist");
        
        // Clean up: remove the test file
        let _ = std::fs::remove_file("test_report.pdf");
    }
}

