use axum::{
    extract::DefaultBodyLimit,
    http::{header::CONTENT_TYPE, HeaderValue, Method},
    middleware,
    response::Html,
    routing::{get, post},
    Router,
};
use dataset_viewer_backend::{
    api,
    config::Config,
    state::AppState,
    websocket,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "dataset_viewer_backend=info".into()),
        )
        .init();

    // 加载配置
    let config = Config::new()?;
    info!("服务器配置: {:?}", config);

    // 初始化应用状态
    let state = Arc::new(AppState::new(config.clone()).await?);

    // 配置 CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE])
        .allow_credentials(true)
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap()); // Vue 开发服务器地址

    // 构建路由
    let app = Router::new()
        // 健康检查
        .route("/health", get(|| async { "OK" }))
        
        // 静态首页
        .route("/", get(|| async { 
            Html(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Dataset Viewer Backend</title>
                <meta charset="utf-8">
            </head>
            <body>
                <h1>Dataset Viewer Backend API</h1>
                <p>服务器正在运行中...</p>
                <p>API 文档: <a href="/api/docs">/api/docs</a></p>
                <p>WebSocket 连接: <code>ws://localhost:8080/ws</code></p>
            </body>
            </html>
            "#)
        }))

        // API 路由
        .nest("/api", api::routes())
        
        // WebSocket 路由
        .route("/ws", get(websocket::handler))

        // 应用中间件
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(1024 * 1024 * 100)) // 100MB 限制
        .with_state(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("🚀 Dataset Viewer Backend 启动在 http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}