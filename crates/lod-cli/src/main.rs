use anyhow::Result;
use clap::{Parser, Subcommand};
use lod_core::*;

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
        Commands::Validate { data, shapes, report } => {
            let r = lod.validate(ValidationRequest {
                data_graph_path: data,
                shapes_graph_path: shapes,
                report_path: report,
            })?;
            println!("Conforms: {}", r.conforms);
            for issue in r.issues {
                println!(
                    "[{}] {}{}",
                    issue.severity,
                    issue.message,
                    issue.line.map(|l| format!(" (line {l})")).unwrap_or_default()
                );
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
