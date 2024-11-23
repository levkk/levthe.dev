mod controllers;
mod models;

use rwf::controller::StaticFiles;
use rwf::http::{self, Server};
use rwf::prelude::*;

#[derive(Default)]
struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, _: &Request) -> Result<Response, Error> {
        Ok(Response::new().redirect("/articles/"))
    }
}

#[derive(Default)]
struct NotFound;

#[async_trait]
impl Controller for NotFound {
    async fn handle(&self, _: &Request) -> Result<Response, Error> {
        render!("templates/not_found.html",
            "title" => "Page not found | Lev's blog",
            404
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    Logger::init();

    Server::new(vec![
        route!("/" => Index),
        route!("/blog/:page" => controllers::content::Content),
        route!("/articles" => controllers::articles::Articles),
        StaticFiles::serve("static")?,
        NotFound::default().wildcard("/"),
    ])
    .launch("0.0.0.0:8000")
    .await
}
