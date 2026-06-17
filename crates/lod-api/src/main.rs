use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use lod_core::{parser, LodGraph, LodWorkbench, RdfFormat, VisualizationEdge, VisualizationGraph, VisualizationNode};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    lod: Arc<LodWorkbench>,
}

#[derive(Debug, Deserialize)]
struct TextGraphRequest {
    content: String,
    format: String,
}

#[derive(Debug, Deserialize)]
struct ConvertTextRequest {
    content: String,
    from: String,
    to: String,
}

#[derive(Debug, Serialize)]
struct ConvertTextResponse {
    output: String,
}

#[derive(Debug, Serialize)]
struct VisualizeTextResponse {
    graph: VisualizationGraph,
    jsonld: String,
    triples: usize,
}

#[derive(Debug, Serialize)]
struct ApiErrorResponse {
    code: String,
    error: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let app = build_app();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("LOD Workbench API listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_app() -> Router {
    let state = AppState {
        lod: Arc::new(LodWorkbench::default()),
    };

    // Keep the browser UI and the raw API in the same router so both share
    // the same CORS policy and application state.
    Router::new()
        .route("/", get(index))
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/inspect-text", post(inspect_text))
        .route("/api/validate-text", post(validate_text))
        .route("/api/convert-text", post(convert_text))
        .route("/api/visualize-text", post(visualize_text))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn index() -> Html<&'static str> {
    Html(
        r#"<!doctype html><html><head><meta charset='utf-8'><title>LOD Workbench API</title></head><body><h1>LOD Workbench API</h1><p>Use the React app in <code>apps/web</code> or POST to <code>/api/inspect-text</code>, <code>/api/convert-text</code>, <code>/api/validate-text</code>.</p></body></html>"#,
    )
}

async fn inspect_text(State(state): State<AppState>, Json(req): Json<TextGraphRequest>) -> impl IntoResponse {
    let fmt = match RdfFormat::parse(&req.format) {
        Ok(f) => f,
        Err(e) => return api_error(StatusCode::BAD_REQUEST, e.to_string()),
    };
    match state.lod.inspect_content(&req.content, fmt, None) {
        Ok(report) => Json(report).into_response(),
        Err(e) => api_error(StatusCode::BAD_REQUEST, e.to_string()),
    }
}

async fn validate_text(State(state): State<AppState>, Json(req): Json<TextGraphRequest>) -> impl IntoResponse {
    let fmt = match RdfFormat::parse(&req.format) {
        Ok(f) => f,
        Err(e) => return api_error(StatusCode::BAD_REQUEST, e.to_string()),
    };
    match state.lod.validate_content(&req.content, fmt, None, None) {
        Ok(report) => Json(report).into_response(),
        Err(e) => api_error(StatusCode::BAD_REQUEST, e.to_string()),
    }
}

async fn convert_text(Json(req): Json<ConvertTextRequest>) -> impl IntoResponse {
    let from = match RdfFormat::parse(&req.from) {
        Ok(f) => f,
        Err(e) => return api_error(StatusCode::BAD_REQUEST, e.to_string()),
    };
    let to = match RdfFormat::parse(&req.to) {
        Ok(f) => f,
        Err(e) => return api_error(StatusCode::BAD_REQUEST, e.to_string()),
    };
    match parser::parse_graph(&req.content, from).and_then(|g| parser::serialize_graph(&g, to)) {
        Ok(output) => Json(ConvertTextResponse { output }).into_response(),
        Err(e) => api_error(StatusCode::BAD_REQUEST, e.to_string()),
    }
}

async fn visualize_text(Json(req): Json<TextGraphRequest>) -> impl IntoResponse {
    let fmt = match RdfFormat::parse(&req.format) {
        Ok(f) => f,
        Err(e) => return api_error(StatusCode::BAD_REQUEST, e.to_string()),
    };
    match parser::parse_graph(&req.content, fmt) {
        Ok(graph) => {
            let preview = build_visualization_graph(&graph);
            let jsonld = parser::serialize_graph(&graph, RdfFormat::JsonLd)
                .unwrap_or_else(|_| "{\"@context\":{},\"@graph\":[]}".into());
            Json(VisualizeTextResponse {
                graph: preview,
                jsonld,
                triples: graph.triples.len(),
            })
            .into_response()
        }
        Err(e) => api_error(StatusCode::BAD_REQUEST, e.to_string()),
    }
}

fn build_visualization_graph(graph: &LodGraph) -> VisualizationGraph {
    let subjects: BTreeSet<String> = graph.triples.iter().map(|t| node_label(&t.subject)).collect();
    let mut nodes = std::collections::BTreeMap::new();
    let mut edges = Vec::new();

    // Deduplicate nodes by label so the browser graph stays compact and we do
    // not render one node per triple occurrence.
    for (i, t) in graph.triples.iter().enumerate() {
        let s_label = node_label(&t.subject);
        let o_label = node_label(&t.object);

        nodes
            .entry(s_label.clone())
            .or_insert_with(|| make_visualization_node(&s_label, &t.subject, true));
        nodes
            .entry(o_label.clone())
            .or_insert_with(|| make_visualization_node(&o_label, &t.object, subjects.contains(&o_label)));

        edges.push(VisualizationEdge {
            id: format!("e{i}"),
            source: s_label,
            target: o_label,
            label: short(&t.predicate),
        });
    }

    VisualizationGraph {
        nodes: nodes.into_values().collect(),
        edges,
    }
}

fn node_label(n: &lod_core::Node) -> String {
    match n {
        lod_core::Node::Iri(i) => i.clone(),
        lod_core::Node::Blank(b) => b.clone(),
        lod_core::Node::Literal { value, .. } => format!("literal:{value}"),
    }
}

fn short(s: &str) -> String {
    s.rsplit(['#', '/']).next().unwrap_or(s).chars().take(42).collect()
}

fn make_visualization_node(id: &str, node: &lod_core::Node, has_outgoing: bool) -> VisualizationNode {
    let (node_type, color, shape) = match node {
        lod_core::Node::Iri(_) => ("iri", "#4f46e5", "ellipse"),
        lod_core::Node::Blank(_) => ("blank", "#d97706", "diamond"),
        lod_core::Node::Literal { .. } => {
            // Literal nodes are treated like hubs when they fan out to support
            // list and bag structures in the visualization.
            if has_outgoing {
                ("literal-hub", "#059669", "round-rectangle")
            } else {
                ("literal-leaf", "#059669", "round-rectangle")
            }
        }
    };

    VisualizationNode {
        id: id.to_string(),
        label: short(id),
        node_type: node_type.to_string(),
        color: color.to_string(),
        shape: shape.to_string(),
    }
}

fn api_error(code: StatusCode, error: String) -> axum::response::Response {
    (
        code,
        Json(ApiErrorResponse {
            code: code.as_str().to_string(),
            error,
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use lod_core::{InspectionReport, ValidationReport};

    #[tokio::test]
    async fn visualize_text_returns_graph_payload() {
        let response = visualize_text(Json(TextGraphRequest {
            content: "@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" .\n".into(),
            format: "turtle".into(),
        }))
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(payload["triples"], 1);
        assert!(payload["graph"]["nodes"].is_array());
        assert!(payload["graph"]["edges"].is_array());
    }

    #[tokio::test]
    async fn inspect_text_uses_content_without_temp_files() {
        let response = inspect_text(
            State(AppState {
                lod: Arc::new(LodWorkbench::default()),
            }),
            Json(TextGraphRequest {
                content: "@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" .\n".into(),
                format: "turtle".into(),
            }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: InspectionReport = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(payload.triples, 1);
    }

    #[tokio::test]
    async fn validate_text_returns_report() {
        let response = validate_text(
            State(AppState {
                lod: Arc::new(LodWorkbench::default()),
            }),
            Json(TextGraphRequest {
                content: "@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" .\n".into(),
                format: "turtle".into(),
            }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: ValidationReport = serde_json::from_slice(&bytes).unwrap();
        assert!(payload.conforms);
    }

    #[tokio::test]
    async fn convert_text_returns_serialized_output() {
        let response = convert_text(Json(ConvertTextRequest {
            content: "@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" .\n".into(),
            from: "turtle".into(),
            to: "json-ld".into(),
        }))
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(payload["output"].as_str().unwrap().contains("\"@graph\""));
    }
}
