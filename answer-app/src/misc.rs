use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref TEXT: HashMap<&'static str, [&'static str; 2]> = HashMap::from([
        ("username_len", ["Username should be longer than 5 symbols", "Юзернейм должен быть длинее 5 символов"]),
        ("password_len", ["Password should be longer than 8 symbols", "Паролб должен быть длинее 8 символов"]),
        ("username_cont", ["Username should consist only from alphanumeric symbols", "Юзернейм должен состоять лишь из букв и чисел"]),
        ("password_cont", ["Password should consist only from alphanumeric symbols + !@#$%^&*()_+=-?><", "Пароль должен состоять лишь из букв и чисел + !@#$%^&*()_+=-?><"]),
        ("username_registered", ["Username is already registered", "Юзернейм уже зарегистрирован"]),
    ]);

    pub static ref LANG: HashMap<&'static str, usize> = HashMap::from([
        ("EN", 0),
        ("RU", 1),
    ]);
}

pub async fn validate(username: &String, password: &String, language: &str) -> Result<(), Vec<String>> {
    let mut errs: Vec<String> = vec![];
    let lang_id: usize = LANG[language];
    if username.len() < 5 {
        errs.push(TEXT["username_len"][lang_id].to_string());
    }
    if password.len() < 8 {
        errs.push(TEXT["password_len"][lang_id].to_string());
    }
    if !username.chars().all(char::is_alphanumeric) {
        errs.push(TEXT["username_cont"][lang_id].to_string());
    }
    if !password.chars().all(|x| char::is_alphanumeric(x) || "!@#$%^&*()_+=-?><".contains(x)) {
        errs.push(TEXT["password_cont"][lang_id].to_string());
    }
    
    if errs.is_empty() {
        return Ok(());
    } else {
        return Err(errs);
    }
}
