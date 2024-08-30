use std::{collections::HashMap, time};

use actix_web_validator::Json;
use awc::Client;
use serde::{Deserialize, Serialize};
use skynet_api::{
    actix_cloud::{
        actix_web::{
            cookie::{time::Duration, Cookie, SameSite},
            web::Data,
            Responder,
        },
        bail,
        response::{JsonResponse, RspResult},
        session::Session,
        state::GlobalState,
    },
    anyhow, finish,
    request::Request,
    sea_orm::{DatabaseConnection, TransactionTrait},
    tracing::{info, warn},
    HyUuid, Result, Skynet,
};
use validator::Validate;

use crate::{
    finish_data, finish_err, finish_ok,
    request::{new_csrf_token, APIError, CSRF_COOKIE},
    SkynetResponse,
};

#[derive(Debug, Validate, Deserialize)]
pub struct SigninReq {
    #[validate(length(max = 32))]
    pub username: String,
    pub password: String,
    pub remember: Option<bool>,
    #[serde(rename = "g-recaptcha-response")]
    pub recaptcha: Option<String>,
}

pub async fn signin(
    param: Json<SigninReq>,
    db: Data<DatabaseConnection>,
    session: Session,
    req: Request,
    skynet: Data<Skynet>,
) -> RspResult<impl Responder> {
    if skynet.config.recaptcha.enable {
        if let Some(x) = &param.recaptcha {
            let timeout = skynet.config.recaptcha.timeout;
            if verify_recaptcha(
                x.to_owned(),
                req.extension.real_ip.ip().to_string(),
                RecaptchaOption {
                    url: skynet.config.recaptcha.url.clone(),
                    secret: skynet.config.recaptcha.secret.clone().unwrap(),
                    timeout: if timeout == 0 {
                        None
                    } else {
                        Some(time::Duration::from_secs(timeout.into()))
                    },
                },
            )
            .await
            .is_err()
            {
                finish_err!(SkynetResponse::RecaptchaInvalid);
            }
        } else {
            finish!(JsonResponse::bad_request(
                APIError::MissingField(String::from("recaptcha")).to_string()
            ));
        }
    }

    let tx = db.begin().await?;
    let (ok, user) = skynet
        .user
        .check_pass(&tx, &param.username, &param.password)
        .await?;
    if !ok {
        warn!(username = param.username, "Invalid username or password");
        finish_err!(SkynetResponse::UserInvalid);
    }
    let user = skynet
        .user
        .update_login(
            &tx,
            &user.unwrap().id,
            &req.extension.real_ip.ip().to_string(),
        )
        .await?;
    tx.commit().await?;

    session.renew();
    session.insert("_id", user.id)?;
    session.insert("name", user.username.clone())?;
    session.insert("time", user.last_login.unwrap())?;
    if param.remember.is_some_and(|x| x) {
        session.insert("_ttl", skynet.config.session.remember)?;
    } else {
        session.insert("_ttl", skynet.config.session.expire)?;
    }
    info!(success = true, id = %user.id, name = user.username, "User signin");
    finish_ok!();
}

#[derive(Debug)]
struct RecaptchaOption {
    url: String,
    secret: String,
    timeout: Option<time::Duration>,
}

async fn verify_recaptcha(response: String, ip: String, option: RecaptchaOption) -> Result<()> {
    #[derive(Deserialize, Serialize)]
    struct Response {
        success: bool,
        #[serde(default)]
        challenge_ts: String,
        #[serde(default)]
        hostname: String,
        #[serde(default, rename = "error-codes")]
        error_codes: Vec<String>,
    }
    let client = Client::default();
    let mut req = client.post(option.url + "/recaptcha/api/siteverify");
    if let Some(x) = option.timeout {
        req = req.timeout(x);
    }
    let mut rsp = req
        .send_form(&[
            ("secret", option.secret),
            ("remoteip", ip),
            ("response", response),
        ])
        .await
        .map_err(|x| anyhow!(x.to_string()))?;
    let rsp = rsp.json::<Response>().await?;
    if !rsp.error_codes.is_empty() {
        bail!("Remote error codes: {:?}", rsp.error_codes)
    }
    if !rsp.success {
        bail!("Invalid challenge solution or remote IP")
    }
    Ok(())
}

pub async fn signout(session: Session) -> RspResult<impl Responder> {
    session.purge();
    finish_ok!();
}

pub async fn get_access(req: Request) -> RspResult<impl Responder> {
    #[derive(Serialize)]
    struct Rsp {
        signin: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<HyUuid>,
        permission: HashMap<String, i32>,
    }
    let mut rsp = Rsp {
        signin: false,
        id: None,
        permission: HashMap::new(),
    };
    if let Some(id) = req.uid {
        rsp.signin = true;
        rsp.id = Some(id);
    }
    req.perm.into_iter().for_each(|(_, v)| {
        rsp.permission.insert(v.name, v.perm);
    });
    finish_data!(rsp);
}

pub async fn get_token(
    skynet: Data<Skynet>,
    state: Data<GlobalState>,
) -> RspResult<impl Responder> {
    let token = new_csrf_token(&skynet, &state).await?;
    finish!(
        JsonResponse::new(SkynetResponse::Success).builder(move |r| {
            r.cookie(
                Cookie::build(CSRF_COOKIE, &token)
                    .max_age(Duration::seconds(skynet.config.csrf.expire.into()))
                    .http_only(false)
                    .path("/")
                    .same_site(SameSite::Strict)
                    .secure(skynet.config.listen.ssl)
                    .finish(),
            );
        })
    );
}
