mod tools;
use dotenv::dotenv;
use mcp_core::{server::Server, transport::ServerSseTransport, types::ServerCapabilities};
use serde_json::json;
use std::env;

#[cfg(any(feature = "huggingface", feature = "replicate"))]
use tools::*;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

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
            replicate::GetModelInfoTool::tool(),
            replicate::GetModelInfoTool::call(),
        )
        .register_tool(
            replicate::GetPredictionTool::tool(),
            replicate::GetPredictionTool::call(),
        );

    Server::start(ServerSseTransport::new(
        "0.0.0.0".to_string(),
        std::env::var("SERVER_PORT")
            .ok()
            .and_then(|port| port.parse::<u16>().ok())
            .unwrap_or_else(|| panic!("SERVER_PORT must be a valid int")),
        server_protocol_builder.build(),
    ))
    .await
}
