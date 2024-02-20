# actix-sitemap-rs

## Install 
```bash
cargo install actix-sitemap-rs
```

## Use 
```rust
use actix_sitemaps_rs::{serve_sitemap, ShowErrorMessageStrategy, SitemapBuilder};
use actix_web::{web::Data, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    // Setup sitemap 
    let sitemap = SitemapBuilder::default()
        .static_file("./sitemaps.xml".to_string())
        .web_directory(".well-known".to_string())
        .web_filename("sitemaps.xml".to_string())
        .not_found_strategy(ShowErrorMessageStrategy) // or RedirectToRootStrategy
        .build();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(sitemap.clone()))
            .service(serve_sitemap) // Declare Sitemap as a service
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```