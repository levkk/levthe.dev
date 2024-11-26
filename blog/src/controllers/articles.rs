use rwf::prelude::*;
use tokio::fs::read_dir;

#[derive(Default)]
pub struct Articles;

impl Articles {
    pub async fn articles() -> Result<Vec<String>, Error> {
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

        Ok(entries)
    }
}

#[async_trait]
impl Controller for Articles {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let entries = Self::articles().await?;

        let mut path = request.path().path().to_owned();
        let canonical = if path.ends_with("/") {
            path.pop();
            path
        } else {
            path
        };

        render!(request,
            "templates/articles.html",
            "title" => "Articles | Lev's blog",
            "articles" => entries,
            "canonical" => canonical,
        )
    }
}
