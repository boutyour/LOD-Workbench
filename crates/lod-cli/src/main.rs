use anyhow::Result;
use clap::{Parser, Subcommand};
use lod_core::{
    ConversionRequest, InspectionRequest, LodWorkbench, MappingRequest, RdfFormat, ValidationReportFormat,
    ValidationRequest, VisualizationRequest,
};

#[derive(Parser)]
#[command(name = "lod")]
#[command(version)]
#[command(
    about = "LOD Workbench CLI: conversion, validation, inspection, mapping and visualization for Linked Open Data"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Convert {
        input: String,
        output: String,
        #[arg(long = "from")]
        from_format: Option<String>,
        #[arg(long = "to")]
        to_format: Option<String>,
    },
    Inspect {
        input: String,
        #[arg(long)]
        format: Option<String>,
        #[arg(long)]
        json: Option<String>,
    },
    Validate {
        data: String,
        shapes: Option<String>,
        #[arg(long)]
        report: Option<String>,
        #[arg(long = "report-format")]
        report_format: Option<String>,
    },
    Shacl {
        data: String,
        shapes: String,
        #[arg(long)]
        report: Option<String>,
        #[arg(long = "report-format")]
        report_format: Option<String>,
    },
    Map {
        input: String,
        mapping: String,
        output: String,
        #[arg(long = "to")]
        to_format: Option<String>,
    },
    Visualize {
        input: String,
        #[arg(long)]
        format: Option<String>,
        #[arg(long)]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let lod = LodWorkbench::default();
    match cli.command {
        Commands::Convert {
            input,
            output,
            from_format,
            to_format,
        } => {
            lod.convert(ConversionRequest {
                input_path: input,
                output_path: output.clone(),
                source_format: from_format,
                target_format: to_format,
            })?;
            println!("Converted RDF written to {output}");
        }
        Commands::Inspect { input, format, json } => {
            let report = lod.inspect(InspectionRequest {
                input_path: input,
                input_format: format,
                json_output: json,
            })?;
            println!("Triples:      {}", report.triples);
            println!("Subjects:     {}", report.subjects);
            println!("Predicates:   {}", report.predicates);
            println!("Objects:      {}", report.objects);
            println!("IRIs:         {}", report.iris);
            println!("Literals:     {}", report.literals);
            println!("Blank nodes:  {}", report.blank_nodes);
            println!("Classes:      {}", report.classes);
            println!("Properties:   {}", report.properties);
        }
        Commands::Validate {
            data,
            shapes,
            report,
            report_format,
        } => {
            let report_format = match report_format {
                Some(format) => Some(parse_report_format(&format)?),
                None => None,
            };
            let r = lod.validate(ValidationRequest {
                data_graph_path: data,
                shapes_graph_path: shapes,
                report_path: report,
                report_format,
            })?;
            println!("Conforms: {}", r.conforms);
            for issue in r.issues {
                println!(
                    "[{}] {}{}{}{}{}",
                    issue.severity,
                    issue.message,
                    issue.line.map(|l| format!(" (line {l})")).unwrap_or_default(),
                    issue.column.map(|c| format!(":{c}")).unwrap_or_default(),
                    issue.token.map(|t| format!(" [token: {t}]")).unwrap_or_default(),
                    issue.suggestion.map(|s| format!(" [hint: {s}]")).unwrap_or_default()
                );
                if let Some(node) = issue.focus_node {
                    println!("    node: {node}");
                }
                if let Some(component) = issue.constraint_component {
                    println!("    constraint: {component}");
                }
                if let Some(path) = issue.path {
                    println!("    path: {path}");
                }
                if let Some(value) = issue.value {
                    println!("    value: {value}");
                }
                if let Some(source_shape) = issue.source_shape {
                    println!("    source shape: {source_shape}");
                }
                if let Some(details) = issue.details {
                    println!("    details: {details}");
                }
            }
        }
        Commands::Shacl {
            data,
            shapes,
            report,
            report_format,
        } => {
            let data_content = std::fs::read_to_string(&data)?;
            let data_format = RdfFormat::from_path(&data)?;
            let shapes_content = std::fs::read_to_string(&shapes)?;
            let shapes_format = RdfFormat::from_path(&shapes)?;
            let report_format = match report_format {
                Some(format) => Some(parse_report_format(&format)?),
                None => None,
            };
            let r = lod.validate_content_with_shapes_report(
                &data_content,
                data_format,
                Some(&shapes_content),
                Some(shapes_format),
                report,
                report_format,
            )?;
            println!("Conforms: {}", r.conforms);
            for issue in r.issues {
                println!(
                    "[{}] {}{}{}{}{}",
                    issue.severity,
                    issue.message,
                    issue.line.map(|l| format!(" (line {l})")).unwrap_or_default(),
                    issue.column.map(|c| format!(":{c}")).unwrap_or_default(),
                    issue.token.map(|t| format!(" [token: {t}]")).unwrap_or_default(),
                    issue.suggestion.map(|s| format!(" [hint: {s}]")).unwrap_or_default()
                );
                if let Some(node) = issue.focus_node {
                    println!("    node: {node}");
                }
                if let Some(component) = issue.constraint_component {
                    println!("    constraint: {component}");
                }
                if let Some(path) = issue.path {
                    println!("    path: {path}");
                }
                if let Some(value) = issue.value {
                    println!("    value: {value}");
                }
                if let Some(source_shape) = issue.source_shape {
                    println!("    source shape: {source_shape}");
                }
                if let Some(details) = issue.details {
                    println!("    details: {details}");
                }
            }
        }
        Commands::Map {
            input,
            mapping,
            output,
            to_format,
        } => {
            lod.map_csv_to_rdf(MappingRequest {
                input_path: input,
                mapping_path: mapping,
                output_path: output.clone(),
                output_format: to_format,
            })?;
            println!("Mapped RDF written to {output}");
        }
        Commands::Visualize { input, format, output } => {
            lod.visualize(VisualizationRequest {
                input_path: input,
                input_format: format,
                output_path: output.clone(),
            })?;
            println!("Visualization written to {output}");
        }
    }
    Ok(())
}

fn parse_report_format(value: &str) -> Result<ValidationReportFormat> {
    match value.to_ascii_lowercase().as_str() {
        "html" => Ok(ValidationReportFormat::Html),
        "json" => Ok(ValidationReportFormat::Json),
        "text" | "txt" => Ok(ValidationReportFormat::Text),
        other => Err(anyhow::anyhow!("unsupported report format: {other}")),
    }
}
