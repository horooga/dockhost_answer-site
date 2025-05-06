use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder, Result,
    error, get, post,
    http::{StatusCode, header::ContentType},
    middleware::Logger,
    web::{Html, Data, Form},
    cookie::{Cookie, SameSite},
};
use actix_files::{Files, NamedFile};
use tera::Tera;
use std::{env, io::Write, path::PathBuf};
use time::{Duration, OffsetDateTime};
use jsonwebtoken::{encode, decode, Header, EncodingKey, };
mod auth;
use auth::*;

fn make_jwt(username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = OffsetDateTime::now_utc() + Duration::hours(1);
    let claims = Claims {
        usrnm: username.to_owned(),
        exp: expiration.unix_timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(&JWT_SECRET))
}

#[post("/user-login")]
async fn user_login(Form(form): Form<UserLogin>) -> impl Responder {
    match make_jwt(&form.username) {
        Ok(token) => {
            let cookie = Cookie::build("token", token)
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax)
                .max_age(time::Duration::hours(1))
                .finish();

            HttpResponse::Found()
                .append_header(("Location", "/profile"))
                .cookie(cookie)
                .finish()
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to create token"),
    }
}

#[post("/user-register")]
async fn user_register(Form(form): Form<UserLogin>) -> impl Responder {
    match make_jwt(&form.username) {
        Ok(token) => {
            let cookie = Cookie::build("token", token)
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax)
                .max_age(time::Duration::hours(1))
                .finish();

            HttpResponse::Found()
                .append_header(("Location", "/profile"))
                .cookie(cookie)
                .finish()
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to create token"),
    }
}

#[get("/")]
async fn index() -> impl Responder {
    let path: PathBuf = "./static/html/start.html".parse().unwrap();
    return NamedFile::open(path).unwrap();
}

#[get("/login")]
async fn login() -> impl Responder {
    let path: PathBuf = "./static/html/login.html".parse().unwrap();
    return NamedFile::open(path).unwrap();
}

#[get("/register")]
async fn register() -> impl Responder {
    let path: PathBuf = "./static/html/register.html".parse().unwrap();
    return NamedFile::open(path).unwrap();
}

#[post("/answer")]
async fn next_question(tmpl: Data<Tera>) -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("answers", &["pivo", "snus", "vkid"]);
    return Html::new(tmpl.render("question.html", &ctx).unwrap());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(|| {
        let tera = Tera::new("/app/static/html/*").unwrap();

        App::new()
            .service(index)
            .service(next_question)
            .service(login)
            .service(register)
            .service(user_register)
            .service(user_login)
            .service(Files::new("/static", "./static"))
            .wrap(Logger::new("%r %s %U").log_target("http_log"))
            .app_data(Data::new(tera))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
