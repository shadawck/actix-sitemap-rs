use actix_web::{
    dev::ResourcePath,
    get,
    http::{self, header::ContentType},
    web, HttpResponse,
};

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

pub trait NotFoundStrategy: Send + Sync {
    fn handle_not_found(&self) -> HttpResponse;
}

pub struct RedirectToRootStrategy;
impl NotFoundStrategy for RedirectToRootStrategy {
    fn handle_not_found(&self) -> HttpResponse {
        let redirect_response = HttpResponse::build(http::StatusCode::PERMANENT_REDIRECT)
            .append_header(("Location", "/"))
            .finish();

        redirect_response
    }
}

pub struct ShowErrorMessageStrategy;
impl NotFoundStrategy for ShowErrorMessageStrategy {
    fn handle_not_found(&self) -> HttpResponse {
        let not_found_response =
            HttpResponse::build(http::StatusCode::NOT_FOUND).body("404 Not Found");

        not_found_response
    }
}

#[derive(Clone)]
pub struct Sitemap {
    pub static_file_path: PathBuf,
    pub web_directory: PathBuf,
    pub web_filename: PathBuf,
    pub not_found_strategy: Arc<Box<dyn NotFoundStrategy>>,
}

pub struct SitemapBuilder {
    pub static_file_path: String,
    pub web_directory: String,
    pub web_filename: String,
    pub not_found_strategy: Box<dyn NotFoundStrategy>,
}

impl Default for SitemapBuilder {
    fn default() -> Self {
        Self {
            static_file_path: String::from("sitemaps.xml"),
            web_directory: String::from(""),
            web_filename: String::from("sitemaps.xml"),
            not_found_strategy: Box::new(ShowErrorMessageStrategy) as Box<dyn NotFoundStrategy>,
        }
    }
}

impl SitemapBuilder {
    pub fn web_directory(mut self, web_directory: String) -> Self {
        self.web_directory = web_directory;
        self
    }

    pub fn static_file(mut self, static_file_path: String) -> Self {
        self.static_file_path = static_file_path;
        self
    }

    pub fn web_filename(mut self, web_filename: String) -> Self {
        self.web_filename = web_filename;
        self
    }

    pub fn not_found_strategy(mut self, strategy: impl NotFoundStrategy + 'static) -> Self {
        self.not_found_strategy = Box::new(strategy);
        self
    }

    pub fn build(self) -> Sitemap {
        Sitemap {
            static_file_path: Path::new(&self.static_file_path).to_path_buf(),
            web_directory: Path::new(&self.web_directory).to_path_buf(),
            web_filename: Path::new(&self.web_filename).to_path_buf(),
            not_found_strategy: Arc::new(self.not_found_strategy),
        }
    }
}

#[get("/{requested_path:.*}")]
pub async fn serve_sitemap(
    requested_path: web::Path<String>,
    data: web::Data<Sitemap>,
) -> HttpResponse {
    let expected_path = data.web_directory.join(data.web_filename.as_path());
    let requested_path = Path::new(requested_path.path()).to_path_buf();

    if requested_path != expected_path {
        let strategy = data.not_found_strategy.clone();
        return strategy.handle_not_found();
    }

    let sitemap_fs =
        fs::read_to_string(&data.static_file_path).expect("Can't open sitemaps file !");
    HttpResponse::Ok()
        .content_type(ContentType::xml())
        .body(sitemap_fs)
}

#[cfg(test)]
mod tests {
    use crate::{serve_sitemap, RedirectToRootStrategy, ShowErrorMessageStrategy, SitemapBuilder};
    use actix_web::web::Data;
    use actix_web::{http::header::ContentType, test, App};

    #[actix_web::test]
    async fn given_sitemap_at_root_then_get_success_status_code_when_show_error_message_strategy() {
        let sitemap = SitemapBuilder::default()
            .static_file("sitemaps.xml".to_string())
            .web_filename("sitemaps.xml".to_string())
            .not_found_strategy(ShowErrorMessageStrategy)
            .build();

        let app = test::init_service(
            App::new()
                .app_data(Data::new(sitemap.clone()))
                .service(serve_sitemap),
        )
        .await;

        let req = test::TestRequest::default()
            .insert_header(ContentType::xml())
            .uri("/sitemaps.xml")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn given_sitemap_with_webdirectory_then_get_success_status_code_when_show_error_message_strategy(
    ) {
        let sitemap = SitemapBuilder::default()
            .static_file("./sitemaps.xml".to_string())
            .web_directory(".well-known/".to_string())
            .web_filename("sitemaps.xml".to_string())
            .not_found_strategy(ShowErrorMessageStrategy)
            .build();

        let app = test::init_service(
            App::new()
                .app_data(Data::new(sitemap.clone()))
                .service(serve_sitemap),
        )
        .await;

        let req = test::TestRequest::default()
            .insert_header(ContentType::xml())
            .uri("/.well-known/sitemaps.xml")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn given_sitemap_then_get_not_found_when_show_error_message_strategy() {
        let sitemap = SitemapBuilder::default()
            .static_file("./sitemaps.xml".to_string())
            .web_directory(".well-known/".to_string())
            .web_filename("sitemaps.xml".to_string())
            .not_found_strategy(ShowErrorMessageStrategy)
            .build();

        let app = test::init_service(
            App::new()
                .app_data(Data::new(sitemap.clone()))
                .service(serve_sitemap),
        )
        .await;

        let req = test::TestRequest::default()
            .insert_header(ContentType::xml())
            .uri("/notfound/sitemaps.xml")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 404);
    }

    #[actix_web::test]
    async fn given_sitemap_then_get_not_found_when_bad_path_and_redirect_to_root_strategy() {
        let sitemap = SitemapBuilder::default()
            .static_file("./sitemaps.xml".to_string())
            .web_directory("./.well-known/".to_string())
            .web_filename("sitemaps.xml".to_string())
            .not_found_strategy(RedirectToRootStrategy)
            .build();

        let app = test::init_service(
            App::new()
                .app_data(Data::new(sitemap.clone()))
                .service(serve_sitemap),
        )
        .await;

        let req = test::TestRequest::default()
            .insert_header(ContentType::xml())
            .uri("/notfound/sitemaps.xml")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.headers().get("location").unwrap(), "/");
    }
}
