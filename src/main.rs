use actix_web::{post, App, HttpServer, HttpResponse, Responder};
use scraper::{Html, Selector};
use std::string::String;
use url::form_urlencoded;
use reqwest::Client;
use embryo::{Embryo, EmbryoList};
use serde_json::from_str;
use std::collections::HashMap;
use std::iter::zip;

static SEARCH_URL: &str = "https://html.duckduckgo.com/html/";
static EXCLUDED_CONTENT: [&str; 0] = [];

#[post("/query")]
async fn query_handler(body: String) -> impl Responder {
    let embryo_list = generate_embryo_list(body).await;
    let response = EmbryoList { embryo_list };
    HttpResponse::Ok().json(response)
}

async fn generate_embryo_list(json_string: String) -> Vec<Embryo> {
    let search: HashMap<String,String> = from_str(&json_string).expect("Erreur lors de la désérialisation JSON");
    let value: String = form_urlencoded::byte_serialize(search.values().next().unwrap().as_bytes()).collect();
    let params : HashMap<String,String>= HashMap::from([("q".to_string(),value.to_string())]);

    let client = Client::new();
    let response = client.post(SEARCH_URL).form(&params).send().await;

    match response {
        Ok(response) => {
            if let Ok(body) = response.text().await {
                let embryo_list = extract_links_from_results(body);
                return embryo_list;
            }
        }
        Err(e) => eprintln!("Error fetching search results: {:?}", e),
    }

    Vec::new()
}

fn extract_links_from_results(html: String) -> Vec<Embryo> {
    let mut embryo_list = Vec::new();
    let fragment = Html::parse_document(&html);
    let url_selector = Selector::parse(r#"a.result__a"#).unwrap();
    let resume_selector = Selector::parse(r#"a.result__snippet"#).unwrap();

    for (url_e, resume_e) in zip(fragment.select(&url_selector), fragment.select(&resume_selector)){
        let url: &str =  url_e.value().attr("href").unwrap();
        let resume: String = resume_e.text().collect::<String>();
         let embryo = Embryo {
             properties: HashMap::from([
                     ("url".to_string(), url.to_string()),
                     ("resume".to_string(),resume.to_string())])
         };

         embryo_list.push(embryo);
    }
    embryo_list
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match em_filter::find_port().await {
        Some(port) => {
            let filter_url = format!("http://localhost:{}/query", port);
            println!("Filter registrer: {}", filter_url);
            em_filter::register_filter(&filter_url).await;
            HttpServer::new(|| App::new().service(query_handler))
                .bind(format!("127.0.0.1:{}", port))?.run().await?;
        },
        None => {
            println!("Can't start");
        },
    }

    Ok(())
}

