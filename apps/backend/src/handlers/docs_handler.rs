use utoipa::OpenApi;

/// OpenAPI specification for the API documentation.
/// Phase 8D: Basic framework with tags defined. Individual handler annotations
/// can be added incrementally in subsequent phases.
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Admin API", description = "管理后台 API"),
        (name = "App API", description = "移动端 / 第三方 App API"),
        (name = "Open API", description = "第三方开放 API（API Key 认证）"),
        (name = "Webhooks", description = "Webhook 管理和投递"),
    )
)]
pub struct ApiDoc;

/// Returns a minimal Swagger UI HTML page that loads the OpenAPI spec from /docs/openapi.json.
/// Uses the CDN-hosted version of Swagger UI.
pub fn swagger_ui_html() -> &'static str {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>Cradle API Docs</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
    SwaggerUIBundle({
        url: "/docs/openapi.json",
        dom_id: '#swagger-ui',
        presets: [
            SwaggerUIBundle.presets.apis,
            SwaggerUIBundle.SwaggerUIStandalonePreset
        ],
        layout: "BaseLayout"
    })
    </script>
</body>
</html>"#
}
