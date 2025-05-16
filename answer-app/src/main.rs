use actix_files::{Files, NamedFile};
use actix_web::{
    App, Either, HttpRequest, HttpResponse, HttpServer, Responder,
    cookie::{Cookie, SameSite},
    get, post,
    web::{Data, Form, Html, Redirect},
};
use deadpool_postgres::{
    Client, Config, ManagerConfig, Pool, RecyclingMethod, tokio_postgres::NoTls,
};
use rand::Rng;
use std::path::PathBuf;
use tera::Tera;
use yaml_rust2::yaml::Yaml;
mod auth;
use auth::{ENV, UserLogin, decode_jwt_from_req, encode_jwt, get_lang_id};
mod misc;
use misc::{ANSWERS, Answer, QUESTIONS, TEXT, validate};
mod db;
use db::{add_user, get_user};

#[post("/user-login")]
async fn user_login(
    pool: Data<Pool>,
    tmpl: Data<Tera>,
    Form(form): Form<UserLogin>,
    req: HttpRequest,
) -> Either<HttpResponse, Html> {
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

            Either::Left(
                HttpResponse::Found()
                    .append_header(("Location", "/profile"))
                    .cookie(cookie)
                    .finish(),
            )
        }
        Err(x) => {
            let mut ctx = tera::Context::new();
            ctx.insert("errors", &x);
            Either::Right(Html::new(tmpl.render("login_errs.html", &ctx).unwrap()))
        }
    };
}

#[post("/user-register")]
async fn user_register(
    pool: Data<Pool>,
    tmpl: Data<Tera>,
    Form(form): Form<UserLogin>,
    req: HttpRequest,
) -> Either<HttpResponse, Html> {
    let lang_id = get_lang_id(req.clone());

    let client: Client = pool.get().await.unwrap();
    if get_user(&client, &form.username, lang_id).await.is_ok() {
        let mut ctx = tera::Context::new();
        ctx.insert("errors", &vec![TEXT["user_registered"][lang_id as usize]]);
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

            Either::Left(
                HttpResponse::Found()
                    .append_header(("Location", "/profile"))
                    .cookie(cookie)
                    .finish(),
            )
        }
        Err(x) => {
            let mut ctx = tera::Context::new();
            ctx.insert("errs", &x);
            Either::Right(Html::new(tmpl.render("register_errs.html", &ctx).unwrap()))
        }
    };
}

#[post("/user-logout")]
async fn user_logout() -> impl Responder {
    let cookie = Cookie::build("token", encode_jwt("None", 0_u8))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::hours(0))
        .finish();

    HttpResponse::Found()
        .append_header(("Location", "/start"))
        .cookie(cookie)
        .finish()
}

#[get("/")]
async fn index() -> impl Responder {
    let path: PathBuf = "./static/html/start.html".parse().unwrap();
    NamedFile::open(path).unwrap()
}

#[get("/login")]
async fn login(req: HttpRequest) -> Either<Redirect, NamedFile> {
    if decode_jwt_from_req(req).is_some() {
        Either::Left(Redirect::to("/profile").see_other())
    } else {
        let path: PathBuf = "./static/html/login.html".parse().unwrap();
        Either::Right(NamedFile::open(path).unwrap())
    }
}

#[get("/register")]
async fn register() -> impl Responder {
    let path: PathBuf = "./static/html/register.html".parse().unwrap();
    NamedFile::open(path).unwrap()
}

#[get("/profile")]
async fn profile(tmpl: Data<Tera>, req: HttpRequest) -> Either<Html, Redirect> {
    let lang_id = get_lang_id(req.clone());

    let mut ctx = tera::Context::new();
    if let Some(jwt) = decode_jwt_from_req(req) {
        ctx.insert("username", &jwt.usrnm);
        ctx.insert("lang", &lang_id);
        Either::Left(Html::new(tmpl.render("profile.html", &ctx).unwrap()))
    } else {
        Either::Right(Redirect::to("/login").see_other())
    }
}

#[get("/answer")]
async fn get_question(tmpl: Data<Tera>, req: HttpRequest) -> Either<Html, Redirect> {
    if req.cookie("token").is_some() {
        let lang_id = get_lang_id(req.clone());
        let mut rng = rand::rng();
        let topic: &str = ["math", "physics"][rng.random_range(0..2)];
        let question: &Yaml = &QUESTIONS[topic][rng.random_range(0..10)];

        let mut ctx = tera::Context::new();
        let options_flag = question["options_flag"].as_bool().unwrap();
        ctx.insert("topic", &topic);
        ctx.insert("question", &question["question"].as_str().unwrap());
        ctx.insert("options_flag", &options_flag);
        if options_flag {
            ctx.insert(
                "options",
                &question["options"]
                    .as_vec()
                    .unwrap()
                    .iter()
                    .map(|s| s.as_str().unwrap())
                    .collect::<Vec<&str>>(),
            );
        }
        Either::Left(Html::new(tmpl.render("question.html", &ctx).unwrap()))
    } else {
        Either::Right(Redirect::to("/login").see_other())
    }
}

#[post("/answer-check")]
async fn check_answer(req: HttpRequest, Form(form): Form<Answer>) -> impl Responder {
    if req.cookie("token").is_some() {
        let lang_id = get_lang_id(req.clone());
        if form.answer == ANSWERS[&form.question.as_str()] {
            Redirect::to("/answer").see_other()
        } else {
            Redirect::to("/login").see_other()
        }
    } else {
        Redirect::to("/login").see_other()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let tera = Tera::new("/app/static/html/*").unwrap();

        let mut cfg = Config::new();
        cfg.dbname = Some("app".to_string());
        cfg.host = Some("postgres".to_string());
        cfg.port = Some(5432);
        cfg.user = Some(ENV["POSTGRES_USER"].clone());
        cfg.password = Some(ENV["POSTGRES_PASSWORD"].clone());
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        let pool = cfg.create_pool(None, NoTls).unwrap();

        App::new()
            .service(index)
            .service(get_question)
            .service(check_answer)
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
