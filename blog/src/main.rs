mod controllers;
mod models;

use std::env::var;

use rwf::controller::{BasicAuth, StaticFiles, TurboStream};
use rwf::http::{self, Server};
use rwf::prelude::*;

#[derive(Default)]
struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, _: &Request) -> Result<Response, Error> {
        Ok(Response::new().redirect("/blog"))
    }
}

#[derive(Default)]
struct NotFound;

#[async_trait]
impl Controller for NotFound {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        render!(
            request,
            "templates/not_found.html",
            "title" => "Page not found | Lev's blog",
            404
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    Logger::init();

    rwf_admin::install()?;

    Migrations::migrate().await?;

    let mut routes = vec![
        route!("/" => Index),
        route!("/blog/:page" => controllers::content::Content),
        route!("/blog" => controllers::articles::Articles),
        route!("/rss.xml" => controllers::rss::Rss),
        route!("/turbo-stream" => TurboStream),
        StaticFiles::serve("static")?,
        NotFound::default().wildcard("/"),
    ];

    let admin_engine = rwf_admin::engine().auth(
        BasicAuth {
            user: var("BLOG_ADMIN_USER").unwrap_or(String::from("admin")),
            password: var("BLOG_ADMIN_PASSWORD").unwrap_or(String::from("admin")),
        }
        .handler(),
    );

    routes.extend(vec![
        engine!("/admin" => admin_engine),
        rwf_admin::static_files()?,
    ]);

    Server::new(routes).launch().await
}
