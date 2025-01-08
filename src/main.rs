use std::{collections::HashSet, usize};

use regex::Regex;
use reqwest::Error;
use sqlx::{pool::PoolOptions, postgres::PgPoolOptions, Acquire};
use url::Url;

async fn get_page(url: &String) -> Result<String, Error> {
    let body = reqwest::get(url).await?.text().await?;
    Ok(body)
}

#[tokio::main]
async fn main() {
    let mut urls: Vec<String> = ["https://wikipedia.org".to_string()].to_vec();
    let db_url = "postgres://rust_user:password@localhost/rust_db";
    let mut visited_urls = HashSet::new();
    for _ in 0..=10 {
        let size = urls.len();
        for i in 0..size {
            let url = &urls[i];
            if visited_urls.contains(url) {
                continue;
            }

            visited_urls.insert(url.clone());

            let size = urls.len();
            println!("{size}");
            match get_page(url).await {
                Ok(content) => {
                    let new_links = extract_links(&content, url).await;
                    urls.extend(new_links);
                }
                Err(e) => {
                    eprintln!("Error trying to get {} : {}", url, e);
                }
            }
        }
    }

    let pages: Vec<Page> = urls
        .iter()
        .enumerate()
        .map(|(id, url)| Page {
            id: (id + 1) as u32,
            url: url.clone(),
        })
        .collect();
    if let Err(e) = manage_db(db_url, &pages).await {
        eprintln!("Error while insering data into db : {}", e);
    }

    println!("finish");
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

struct Page {
    id: u32,
    url: String,
}

async fn manage_db(db_url: &str, pages: &[Page]) -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS websites(
            id SERIAL PRIMARY KEY,
            url TEXT NOT NULL UNIQUE
        );
        "#,
    )
    .execute(&pool)
    .await?;
    let mut transaction = pool.begin().await?;
    let insert_query = r#"
        INSERT INTO websites (url)
        VALUES ($1)
        ON CONFLICT (url) DO NOTHING
    "#;
    for page in pages {
        sqlx::query(insert_query)
            .bind(&page.url)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;
    Ok(())
}
