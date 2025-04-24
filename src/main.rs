mod tools;
use clap::{Parser, ValueEnum};
use dotenv::dotenv;
use mcp_core::{
    server::Server,
    transport::{ServerSseTransport, ServerStdioTransport},
    types::ServerCapabilities,
};
use serde_json::json;
use std::env;

#[cfg(any(feature = "huggingface", feature = "replicate"))]
use tools::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Transport type to use
    #[arg(value_enum, default_value_t = TransportType::Stdio)]
    transport: TransportType,

    /// Optional path to .env file
    #[arg(short, long)]
    env_file: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum TransportType {
    Sse,
    Stdio,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load env file from path if provided, otherwise load from default location
    if let Some(env_path) = Cli::parse().env_file {
        dotenv::from_path(env_path).ok();
    } else {
        dotenv().ok();
    }

    let cli = Cli::parse();

    let server_protocol_builder = Server::builder(
        env::var("SERVER_NAME").expect("SERVER_NAME must be set"),
        env::var("SERVER_VERSION").expect("SERVER_VERSION must be set"),
    )
    .capabilities(ServerCapabilities {
        tools: Some(json!({})),
        ..Default::default()
    });

    #[cfg(feature = "huggingface")]
    let server_protocol_builder = server_protocol_builder
        .register_tool(
            huggingface::GetReadmeTool::tool(),
            huggingface::GetReadmeTool::call(),
        )
        .register_tool(
            huggingface::GetModelInfoTool::tool(),
            huggingface::GetModelInfoTool::call(),
        )
        .register_tool(
            huggingface::GetModelSampleImagesTool::tool(),
            huggingface::GetModelSampleImagesTool::call(),
        )
        .register_tool(
            huggingface::SearchModelsTool::tool(),
            huggingface::SearchModelsTool::call(),
        )
        .register_tool(
            huggingface::WhoamiTool::tool(),
            huggingface::WhoamiTool::call(),
        );

    #[cfg(feature = "replicate")]
    let server_protocol_builder = server_protocol_builder
        .register_tool(
            replicate::ListModelsTool::tool(),
            replicate::ListModelsTool::call(),
        )
        .register_tool(replicate::WhoamiTool::tool(), replicate::WhoamiTool::call())
        .register_tool(
            replicate::GenerateImageTool::tool(),
            replicate::GenerateImageTool::call(),
        )
        .register_tool(
            replicate::EditImageTool::tool(),
            replicate::EditImageTool::call(),
        )
        .register_tool(
            replicate::EditImageWithMaskTool::tool(),
            replicate::EditImageWithMaskTool::call(),
        )
        .register_tool(
            replicate::GetModelInfoTool::tool(),
            replicate::GetModelInfoTool::call(),
        )
        .register_tool(
            replicate::GetPredictionTool::tool(),
            replicate::GetPredictionTool::call(),
        );

    let server = server_protocol_builder.build();

    match cli.transport {
        TransportType::Sse => {
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::DEBUG)
                .init();

            Server::start(ServerSseTransport::new(
                "0.0.0.0".to_string(),
                env::var("SERVER_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .expect("SERVER_PORT must be a valid int"),
                server,
            ))
            .await
        }
        TransportType::Stdio => {
            // Prevents the server from logging to stdout
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::DEBUG)
                .with_writer(std::io::stderr)
                .init();

            Server::start(ServerStdioTransport::new(server)).await
        }
    }
}
