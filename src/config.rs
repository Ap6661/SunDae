use serde::{Deserialize, Serialize};
use subsonic_types::request::Authentication;
use crate::singleton::Singleton;

pub static CONFIG: Singleton<Cfg> = Singleton::new( || { 
    let config: Config = confy::load("sundae", Some("config")).unwrap();
    // Make Random
    let salt = "abcd".to_string();

    let token = format!("{:?}", md5::compute([ 
            config.password.clone(),
            salt.clone() 
    ].join("").as_bytes()));

    let auth = Authentication::Token {
        salt,
        token,
    };

    Cfg { 
        config,
        auth,
    }
});

pub struct Cfg {
    pub config: Config,
    pub auth: Authentication,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub server: String,
    pub username: String,
    pub password: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self { 
        Self { 
            server: String::from("serverurl"), 
            username: String::from("USERNAME"),
            password: String::from("PASSWORD"),
        }
    }

}
