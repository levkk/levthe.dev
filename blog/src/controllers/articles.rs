use rwf::prelude::*;
use tokio::fs::read_dir;

#[derive(Default)]
pub struct Articles;

#[async_trait]
impl Controller for Articles {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let mut directory = read_dir("blog").await?;
        let mut entries = vec![];
        while let Some(entry) = directory.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_dir() {
                continue;
            }

            entries.push(
                entry
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
        }

        entries.sort();

        let mut path = request.path().path().to_owned();
        let canonical = if path.ends_with("/") {
            path
        } else {
            path.push_str("/");
            path
        };

        render!("templates/articles.html",
            "title" => "Articles | Lev's blog",
            "articles" => entries,
            "canonical" => canonical,
        )
    }
}
