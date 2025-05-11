use actix_web::{
    App, Error, HttpResponse, HttpRequest, HttpServer, Responder, Result, Either,
    error, get, post,
    http::{StatusCode, header::ContentType},
    web::{Html, Data, Form, Redirect},
    cookie::{Cookie, SameSite},
};
use actix_files::{Files, NamedFile};
use tera::Tera;
use std::path::PathBuf;
use deadpool_postgres::{Config, Client, Pool, ManagerConfig, RecyclingMethod, tokio_postgres::NoTls};
use env_file_reader::read_file;
mod auth;
use auth::{UserLogin, get_lang_id, encode_jwt, decode_jwt_from_req};
mod misc;
use misc::{TEXT, validate};
mod db;
use db::{User, add_user, get_user};

#[post("/user-login")]
async fn user_login(pool: Data<Pool>, tmpl: Data<Tera>, Form(form): Form<UserLogin>, req: HttpRequest) -> Either<HttpResponse, Html> {
    let lang_id = get_lang_id(req);

    let client: Client = pool.get().await.unwrap();
    return match get_user(&client, &form.username, lang_id).await {
        Ok(_) => {
            let cookie = Cookie::build("token", encode_jwt(&form.username, lang_id))
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax)
                .max_age(time::Duration::hours(1))
                .finish();

            Either::Left(HttpResponse::Found()
                .append_header(("Location", "/profile"))
                .cookie(cookie)
                .finish())
        },
        Err(x) => {
            let mut ctx = tera::Context::new();
            ctx.insert("errors", &x);
            Either::Right(Html::new(tmpl.render("login_errs.html", &ctx).unwrap()))
        },
    }
}

#[post("/user-register")]
async fn user_register(pool: Data<Pool>, tmpl: Data<Tera>, Form(form): Form<UserLogin>, req: HttpRequest) -> Either<HttpResponse, Html> {
    let lang_id = get_lang_id(req.clone());

    let client: Client = pool.get().await.unwrap();
    if get_user(&client, &form.username, lang_id).await.is_ok() {
        let mut ctx = tera::Context::new();
        ctx.insert("errors", &vec![TEXT["username_registered"][lang_id as usize]]);
        return Either::Right(Html::new(tmpl.render("register_errs.html", &ctx).unwrap()));
    }
    return match validate(&form.username, &form.password, lang_id).await {
        Ok(_) => {
            add_user(&client, &form.username, &form.password, lang_id).await;

            let cookie = Cookie::build("token", encode_jwt(&form.username, lang_id))
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax)
                .max_age(time::Duration::hours(1))
                .finish();

            Either::Left(HttpResponse::Found()
                .append_header(("Location", "/profile"))
                .cookie(cookie)
                .finish())
        },
        Err(x) => {
            let mut ctx = tera::Context::new();
            ctx.insert("errors", &x);
            Either::Right(Html::new(tmpl.render("register_errs.html", &ctx).unwrap()))
        },
    }
}

#[post("/user-logout")]
async fn user_logout() -> impl Responder {
    let cookie = Cookie::build("token", encode_jwt("None", 0_u8))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::hours(0))
        .finish();
    
    return HttpResponse::Found()
        .append_header(("Location", "/start"))
        .cookie(cookie)
        .finish()
}

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    let lang_id = get_lang_id(req);

    let cookie = Cookie::build("token", encode_jwt("None", lang_id))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::hours(1))
        .finish();
    
    return HttpResponse::Ok()
        .content_type(ContentType::html())
        .cookie(cookie)
        .body("./static/html/start.html");
}

#[get("/login")]
async fn login(req: HttpRequest) -> Either<Redirect, NamedFile> {
    return match decode_jwt_from_req(req) {
        Some(jwt) => {
            if jwt.usrnm != "None" {
                Either::Left(Redirect::to("/profile").permanent())
            } else {
                let path: PathBuf = "./static/html/login.html".parse().unwrap();
                Either::Right(NamedFile::open(path).unwrap())
            }
        }
        None => {
            let path: PathBuf = "./static/html/login.html".parse().unwrap();
            Either::Right(NamedFile::open(path).unwrap())
        },
    }
}

#[get("/register")]
async fn register(req: HttpRequest) -> impl Responder {
    let path: PathBuf = "./static/html/register.html".parse().unwrap();
    return NamedFile::open(path).unwrap();
}

#[get("/profile")]
async fn profile(tmpl: Data<Tera>, req: HttpRequest) -> Either<Html, Redirect> {
    let lang_id = get_lang_id(req.clone());

    let mut ctx = tera::Context::new();
    if let Some(jwt) = decode_jwt_from_req(req) {  
        ctx.insert("username", &jwt.usrnm);
        ctx.insert("lang", &lang_id);
        return Either::Left(Html::new(tmpl.render("profile.html", &ctx).unwrap()));
    } else {
        return Either::Right(Redirect::to("/login").permanent());
    }
}

#[get("/answer")]
async fn next_question(tmpl: Data<Tera>, req: HttpRequest) -> Either<Html, Redirect> {
    let lang_id = get_lang_id(req.clone());

    return match req.cookie("token") {
        Some(x) => {
            let mut ctx = tera::Context::new();
            ctx.insert("answers", &["pivo", "snus", "vkid"]);
            Either::Left(Html::new(tmpl.render("question.html", &ctx).unwrap()))
        },
        None => Either::Right(Redirect::to("/login").permanent()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(|| {
        let env: std::collections::HashMap<String, String> = read_file(".env").unwrap();

        let tera = Tera::new("/app/static/html/*").unwrap();

        let mut cfg = Config::new();
        cfg.dbname = Some("app".to_string());
        cfg.host = Some("postgres".to_string());
        cfg.port = Some(5432);
        cfg.user = Some(env["POSTGRES_USER"].clone());
        cfg.password = Some(env["POSTGRES_PASSWORD"].clone());
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        let pool = cfg.create_pool(None, NoTls).unwrap();
 
        App::new()
            .service(index)
            .service(next_question)
            .service(login)
            .service(register)
            .service(user_register)
            .service(user_login)
            .service(profile)
            .service(Files::new("/static", "./static"))
            .app_data(Data::new(tera))
            .app_data(Data::new(pool.clone()))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
