use comrak::nodes::NodeValue;
use comrak::{format_html, parse_document, Arena, Options};
use rwf::prelude::*;

use std::path::Path;
use tokio::fs::read_to_string;

#[derive(Default)]
pub struct Content;

#[async_trait]
impl Controller for Content {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let page = request.parameter::<String>("page")?;
        if let Some(page) = page {
            let path = Path::new("blog").join(format!("{}.md", page));

            let mut options = Options::default();
            options.extension.table = true;
            options.extension.autolink = true;
            options.extension.footnotes = true;
            options.extension.greentext = true;

            if let Ok(content) = read_to_string(path).await {
                let arena = Arena::new();
                let root = parse_document(&arena, &content, &options);

                // Figure out the title from the first <h1>.
                let mut title = "Lev's blog".to_string();
                for node in root.descendants() {
                    if let NodeValue::Heading(heading) = node.data.borrow().value {
                        if heading.level == 1 {
                            if let Some(sibling) = node.first_child() {
                                if let NodeValue::Text(ref text) = sibling.data.borrow().value {
                                    title = format!("{} | Lev's blog", text.clone());
                                }
                            }
                        }
                    }
                }

                let mut html = vec![];
                let _ = format_html(root, &options, &mut html);

                let mut path = request.path().path().to_owned();
                let canonical = if path.ends_with("/") {
                    path.pop();
                    path
                } else {
                    path
                };

                let context = context!(
                    "page" => String::from_utf8_lossy(&html).to_string(),
                    "title" => title,
                    "canonical" => canonical,
                    "request" => request.clone(),
                );

                let template = Template::load("templates/blog.html")?.render(&context)?;

                return Ok(Response::new()
                    .html(template)
                    .header("cache-control", "max-age=3600")
                    .header("cache", "public"));
            }
        }

        Ok(Response::not_found())
    }
}
