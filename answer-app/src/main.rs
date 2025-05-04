use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder, Result,
    error, get, post,
    http::{StatusCode, header::ContentType},
    middleware::{self},
    web::{Html, Data},
};
use actix_files::Files;
use tera::Tera;

#[get("/")]
async fn root(tmpl: Data<Tera>) -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("answers", &["pivo", "snus", "vkid"]);
    return Html::new(tmpl.render("answer.html", &ctx).unwrap());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let tera = Tera::new("/app/static/html/*").unwrap();

        App::new()
            .service(root)
            .service(Files::new("/static", "./static"))
            .app_data(Data::new(tera))
            .wrap(middleware::Logger::default())
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
