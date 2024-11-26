use crate::controllers::articles::Articles;
use rwf::prelude::*;

#[derive(Default)]
pub struct Rss;

#[async_trait]
impl Controller for Rss {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let entries = Articles::articles().await?;
        let template = Template::load("templates/rss.xml")?;
        let ctx = context!(
          "articles" => entries,
          "date" => OffsetDateTime::now_utc(),
        );

        Ok(Response::new()
            .html(template.render(&ctx)?)
            .header("Content-Type", "text/xml"))
    }
}
