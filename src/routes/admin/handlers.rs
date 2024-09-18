use axum::{
    Extension,
    Router,
    routing::get,
    response::{
        Html,
        IntoResponse,
    },
};
use crate::startup::AppState;
use crate::template_helpers::{render_content, RenderTemplateParams};
use crate::constants::{
    route_paths,
    html_templates,
};

pub fn routes() -> Router {
    Router::new()
        .route(route_paths::ROOT, get(self::get::index))
}

mod get {
    use super::*;

    pub async fn index(
        Extension(state): Extension<AppState>
    ) -> impl IntoResponse {
        let mut context = tera::Context::new();
        let boo = "ADMIN PAGE";
        context.insert("boo", &boo);

        match render_content(
            &RenderTemplateParams::new(html_templates::ADMIN_INDEX, &state.tera)
            .with_context(&context)
        ) {
            Ok(homepage_template) => Html(homepage_template).into_response(),
            Err(e) => e.into_response()
        }
    }
}
