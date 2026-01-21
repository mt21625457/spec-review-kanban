mod app;
mod auth;
pub mod config;
pub mod db;
pub mod github_app;
pub mod mail;
pub mod r2;
pub mod routes;
mod state;
pub mod validated_where;

use std::sync::OnceLock;

pub use app::Server;
pub use state::AppState;

static INIT_GUARD: OnceLock<sentry::ClientInitGuard> = OnceLock::new();

fn environment() -> &'static str {
    if cfg!(debug_assertions) {
        "dev"
    } else {
        "production"
    }
}

/// 初始化 Remote 服务专用的 Sentry 客户端
///
/// 注意：Remote 服务使用独立的 Sentry DSN，与其他服务不同
pub fn sentry_init_once() {
    INIT_GUARD.get_or_init(|| {
        sentry::init((
            "https://d6e4c45af2b081fadb10fb0ba726ccaf@o4509603705192449.ingest.de.sentry.io/4510305669283920",
            sentry::ClientOptions {
                release: sentry::release_name!(),
                environment: Some(environment().into()),
                ..Default::default()
            },
        ))
    });

    sentry::configure_scope(|scope| {
        scope.set_tag("source", "remote");
    });
}

pub fn configure_user_scope(user_id: uuid::Uuid, username: Option<&str>, email: Option<&str>) {
    let mut sentry_user = sentry::User {
        id: Some(user_id.to_string()),
        ..Default::default()
    };

    if let Some(username) = username {
        sentry_user.username = Some(username.to_string());
    }

    if let Some(email) = email {
        sentry_user.email = Some(email.to_string());
    }

    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry_user));
    });
}
