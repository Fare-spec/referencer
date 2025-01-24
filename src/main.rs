use std::usize;

use regex::Regex;
use reqwest::Error;
use sqlx::pool::PoolOptions;
use url::Url;

async fn get_page(url: &String) -> Result<String, Error> {
    let body = reqwest::get(url).await?.text().await?;
    Ok(body)
}

#[tokio::main]
async fn main() {
    let mut urls: Vec<String> = ["https://wikipedia.org".to_string()].to_vec();
    for _ in 0..=10 {
        let size = urls.len();

        for i in 0..=size {
            let size = urls.len();
            let content = get_page(&urls[i as usize])
                .await
                .expect("failed to reached url");
            urls.extend(extract_links(&content, &urls[i as usize]).await);
            println!("{i} links: {size}, {urls:?}")
        }
    }
}

async fn extract_links(content: &String, url: &String) -> Vec<String> {
    let re = Regex::new(r#"<a\s+(?:[^>]*?\s+)?href="([^"]*)""#).unwrap();
    let base_parsed = Url::parse(url);
    let base_domain = match base_parsed {
        Ok(url) => match url.host_str() {
            Some(domain) => domain.to_string(),
            _none => {
                eprintln!("Erreur: L'URL de base ne contient pas de domaine valide.");
                return Vec::new();
            }
        },
        Err(e) => {
            eprintln!("Erreur lors de l'analyse de l'URL de base: {}", e);
            return Vec::new();
        }
    };

    let mut liens_externes = std::collections::HashSet::new();

    for cap in re.captures_iter(content) {
        if let Some(lien) = cap.get(1).map(|m| m.as_str()) {
            if let Ok(parsed_lien) = Url::parse(lien) {
                if let Some(domaine_lien) = parsed_lien.host_str() {
                    if domaine_lien != base_domain
                        && !domaine_lien.ends_with(&format!(".{}", base_domain))
                    {
                        let truncated_link = format!("{}://{}", parsed_lien.scheme(), domaine_lien);
                        liens_externes.insert(truncated_link);
                    }
                }
            } else {
                continue;
            }
        }
    }

    liens_externes.into_iter().collect()
}

struct pages {
    id: u32,
    url: String,
    keywords: Vec<String>,
}

async fn manage_db(
    db_url: &str,
    urls: &Vec<String>,
    keywords: &Vec<Vec<String>>,
) -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
}
