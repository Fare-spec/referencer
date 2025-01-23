use std::usize;

use regex::Regex;
use reqwest::Error;

async fn get_page(url: &String) -> Result<String, Error> {
    let body = reqwest::get(url).await?.text().await?;
    println!("body = {body:?}");
    Ok(body)
}

#[tokio::main]
async fn main() {
    let mut urls: Vec<String> = ["https://wikipedia.org".to_string()].to_vec();
    for _ in 0..=10 {
        let size = urls.len();

        for i in 0..=size {
            let content = get_page(&urls[i as usize])
                .await
                .expect("failed to reached url");
            urls.extend(extract_links(&content, &urls[i as usize]).await);
            println!("{i}")
        }
    }
}

async fn extract_links(content: &String, url: &String) -> Vec<String> {
    let re = Regex::new(r#"<a\s+href="([^"]*)">"#).unwrap();

    let links: Vec<_> = re
        .captures_iter(content)
        .filter_map(|cap| {
            let link = cap.get(1)?.as_str();
            if !link.contains(url) {
                let truncated_link = link.split('/').take(3).collect::<Vec<_>>().join("/");
                Some(truncated_link.to_string())
            } else {
                None
            }
        })
        .collect();
    links
}
