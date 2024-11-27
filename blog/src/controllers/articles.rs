use rwf::prelude::*;
use time::{macros::format_description, Date, Time};
use tokio::fs::{read_dir, File};
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(PartialEq, Debug, Clone, PartialOrd, Eq, Ord, macros::TemplateValue)]
pub struct Article {
    pub path: String,
    pub pub_date: Option<String>,
    pub date: Option<String>,
}

#[derive(Default)]
pub struct Articles;

impl Articles {
    pub async fn articles() -> Result<Vec<Article>, Error> {
        let mut directory = read_dir("blog").await?;
        let mut entries = vec![];
        let format = format_description!("[year]-[month]-[day]");

        while let Some(entry) = directory.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_dir() {
                continue;
            }

            let file = File::open(entry.path()).await?;
            let mut reader = BufReader::new(file).lines();

            while let Some(line) = reader.next_line().await? {
                if line.trim().is_empty() {
                    continue;
                }

                let (pub_date, date) = match Date::parse(line.trim(), format) {
                    Ok(date) => {
                        let pub_date = OffsetDateTime::now_utc()
                            .replace_date(date)
                            .replace_time(Time::from_hms(0, 0, 0).unwrap());
                        let pub_date = pub_date
                            .format(&time::format_description::well_known::Rfc2822)
                            .unwrap();
                        let date = format!("{} {}, {}", date.month(), date.day(), date.year());
                        (Some(pub_date), Some(date))
                    }
                    Err(_) => (None, None),
                };

                let path = entry
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                entries.push(Article {
                    path,
                    pub_date,
                    date,
                });
                break;
            }
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
            "articles" => entries.into_iter().map(|e| e.path).collect::<Vec<String>>(),
            "canonical" => canonical,
        )
    }
}
