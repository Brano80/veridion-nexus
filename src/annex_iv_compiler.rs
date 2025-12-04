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

    // --- HEADER ---
    // Dark Bar Background
    let header_bar = Line {
        points: vec![
            (Point::new(Mm(0.0), Mm(280.0)), false),
            (Point::new(Mm(210.0), Mm(280.0)), false),
            (Point::new(Mm(210.0), Mm(297.0)), false),
            (Point::new(Mm(0.0), Mm(297.0)), false),
        ],
        is_closed: true,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: false,
    };
    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.2, None))); // Dark Blue/Grey
    current_layer.add_shape(header_bar);

    // Header Text
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("Failed to add font: {:?}", e))?;
    current_layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None))); // White Text
    current_layer.use_text("VERIDION NEXUS | COMPLIANCE REPORT", 18.0, Mm(10.0), Mm(286.0), &font);
    
    let font_reg = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Failed to add font: {:?}", e))?;
    current_layer.use_text("EU AI Act - Annex IV Technical Documentation", 10.0, Mm(10.0), Mm(281.0), &font_reg);

    // --- TABLE HEADERS ---
    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None))); // Black Text
    let y_start = 260.0;
    current_layer.use_text("TIMESTAMP", 9.0, Mm(10.0), Mm(y_start), &font);
    current_layer.use_text("AGENT ACTION", 9.0, Mm(50.0), Mm(y_start), &font);
    current_layer.use_text("QUALIFIED SEAL ID (eIDAS)", 9.0, Mm(110.0), Mm(y_start), &font);

    // Draw Line under header
    current_layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    current_layer.set_outline_thickness(0.5);
    current_layer.add_shape(Line {
        points: vec![(Point::new(Mm(10.0), Mm(y_start - 2.0)), false), (Point::new(Mm(200.0), Mm(y_start - 2.0)), false)],
        is_closed: false,
        has_fill: false,
        has_stroke: true,
        is_clipping_path: false,
    });

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

        // Grid line
        current_layer.set_outline_color(Color::Rgb(Rgb::new(0.8, 0.8, 0.8, None))); // Light Grey
        current_layer.set_outline_thickness(0.3);
        current_layer.add_shape(Line {
            points: vec![(Point::new(Mm(10.0), Mm(y_pos - 4.0)), false), (Point::new(Mm(200.0), Mm(y_pos - 4.0)), false)],
            is_closed: false,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        });

        y_pos -= 10.0;
    }

    // --- FOOTER ---
    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    current_layer.use_text("Generated automatically by Veridion Nexus - Confidential", 8.0, Mm(10.0), Mm(10.0), &font_reg);

    doc.save(&mut BufWriter::new(File::create(output_path).map_err(|e| e.to_string())?)).map_err(|e| e.to_string())?;
    Ok(())
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
            },
            ComplianceRecord {
                timestamp: "2024-01-15 11:45:00".to_string(),
                action_summary: "Data Shredded".to_string(),
                seal_id: "QES_SEAL_def456".to_string(),
                status: "COMPLETED".to_string(),
            },
            ComplianceRecord {
                timestamp: "2024-01-15 12:00:00".to_string(),
                action_summary: "Compliance Check".to_string(),
                seal_id: "QES_SEAL_ghi789".to_string(),
                status: "VERIFIED".to_string(),
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

