use std::{fs::File, io::BufReader, path::Path};

use crate::{api, Cli};
use actix_files::NamedFile;
use actix_session::{
    config::{PersistentSession, TtlExtensionPolicy},
    storage::RedisSessionStore,
    SessionMiddleware,
};
use actix_web::{
    cookie::{time::Duration, Key, SameSite},
    dev::{fn_service, ServerHandle, ServiceRequest, ServiceResponse},
    middleware,
    web::{scope, Data},
    App, HttpMessage, HttpServer,
};
use awc::body::MessageBody;
use chrono::Utc;
use parking_lot::Mutex;
use redis::aio::ConnectionManager;
use rustls::ServerConfig;
use rustls_pemfile::{certs, private_key};
use sea_orm::{DatabaseConnection, TransactionTrait};
use skynet::{
    config, db,
    logger::Logger,
    plugin::PluginManager,
    request::{get_real_ip, RequestExtension},
    HyUuid, Skynet,
};
use skynet_i18n::i18n;
use tracing::{debug, error, info, info_span, warn, Span};
use tracing_actix_web::{RootSpanBuilder, TracingLogger};

pub async fn init_skynet(
    cli: &Cli,
    mut skynet: Skynet,
) -> (Skynet, DatabaseConnection, ConnectionManager) {
    // load config
    skynet.config = config::load_file(cli.config.to_str().unwrap());
    debug!("Config file {:?} loaded", cli.config);

    // locale
    skynet.add_locale(i18n!("locales"));

    // init db
    let db = db::connect(skynet.config.database_dsn.get())
        .await
        .expect("DB connect failed");
    skynet.default_id = db::init(&db, &skynet).await.expect("DB init failed");
    debug!("DB connected");

    // init redis
    let redis = ConnectionManager::new(
        redis::Client::open(skynet.config.redis_dsn.get()).expect("Redis open failed"),
    )
    .await
    .expect("Redis connect failed");
    debug!("Redis connected");

    // init notification
    Logger::set_db(db.clone());
    Logger::write_pending(&db).await;

    // init setting
    let tx = db.begin().await.unwrap();
    skynet.setting.build_cache(&tx).await.unwrap();
    tx.commit().await.unwrap();

    // init menu
    skynet.menu = api::new_menu(&skynet);

    // init plugin
    let mut plugin = PluginManager::new();
    let mut skynet = plugin.load_all(skynet, &cli.plugin);
    skynet.plugin = plugin;
    (skynet, db, redis)
}

fn load_rustls_config<P: AsRef<Path>>(cert: P, key: P) -> ServerConfig {
    let config = ServerConfig::builder().with_no_client_auth();
    let cert_chain = certs(&mut BufReader::new(File::open(cert).unwrap()))
        .map(Result::unwrap)
        .collect();
    let key = private_key(&mut BufReader::new(File::open(key).unwrap()))
        .unwrap()
        .unwrap();
    config.with_single_cert(cert_chain, key).unwrap()
}

fn get_security_header(ssl: bool, csp: String) -> middleware::DefaultHeaders {
    let mut ret = middleware::DefaultHeaders::new()
        .add(("X-Content-Type-Options", "nosniff"))
        .add(("Referrer-Policy", "same-origin"))
        .add(("X-Frame-Options", "DENY"))
        .add(("X-XSS-Protection", "1; mode=block"))
        .add(("Cross-Origin-Opener-Policy", "same-origin"))
        .add(("Referrer-Policy", "same-origin"))
        .add(("Content-Security-Policy", csp));
    if ssl {
        ret = ret.add(("Strict-Transport-Security", "max-age=31536000; preload"));
    }
    ret
}

fn get_session_middleware(
    s: &Skynet,
    redis: &ConnectionManager,
) -> SessionMiddleware<RedisSessionStore> {
    let cookie_prefix = s.config.session_prefix.get().to_owned();
    let cookie_fn = move |x: &str| format!("{cookie_prefix}{x}");
    SessionMiddleware::builder(
        RedisSessionStore::builder(redis.clone())
            .cache_keygen(cookie_fn.clone())
            .build()
            .unwrap(),
        Key::from(s.config.session_key.get().as_bytes()),
    )
    .cookie_name(s.config.session_cookie.get().to_owned())
    .cookie_secure(s.config.listen_ssl.get())
    .cookie_same_site(SameSite::Strict)
    .session_lifecycle(
        PersistentSession::default()
            .session_ttl_extension_policy(TtlExtensionPolicy::OnEveryRequest)
            .session_ttl(Duration::seconds(s.config.session_expire.get())),
    )
    .build()
}

#[derive(Default)]
pub struct StopHandle {
    inner: Mutex<Option<ServerHandle>>,
}

impl StopHandle {
    /// Sets the server handle to stop.
    pub fn register(&self, handle: ServerHandle) {
        *self.inner.lock() = Some(handle);
    }

    /// Sends stop signal through contained server handle.
    pub fn stop(&self, graceful: bool) {
        #[allow(clippy::let_underscore_future)]
        let _ = self.inner.lock().as_ref().unwrap().stop(graceful);
    }
}

pub struct SpanBuilder;

impl RootSpanBuilder for SpanBuilder {
    fn on_request_start(req: &ServiceRequest) -> Span {
        let start_time = Utc::now();
        let request_id = HyUuid::new();
        req.extensions_mut().insert(RequestExtension {
            start_time,
            request_id,
            ..Default::default()
        });
        let skynet = req.app_data::<Data<Skynet>>().unwrap();
        let user_agent = req
            .headers()
            .get("User-Agent")
            .map_or("", |h| h.to_str().unwrap_or(""));
        let span = info_span!("HTTP request", request_id = %request_id, ip = %get_real_ip(req.request(), skynet));
        span.in_scope(|| {
            info!(
                _time = start_time.timestamp_micros(),
                method = %req.method(),
                path = req.path(),
                user_agent = user_agent,
                "Request start"
            );
        });
        span
    }

    fn on_request_end<B: MessageBody>(
        _: Span,
        outcome: &Result<ServiceResponse<B>, actix_web::Error>,
    ) {
        match &outcome {
            Ok(response) => {
                let req = response.request().extensions();
                let req = req.get::<RequestExtension>().unwrap();
                let end_time = Utc::now();
                let time = (end_time - req.start_time).num_microseconds().unwrap_or(0);
                if let Some(error) = response.response().error() {
                    info!(
                        _time = end_time.timestamp_micros(),
                        code = %response.status().as_u16(),
                        error = %error.as_response_error(),
                        process_time = time,
                        "Request end"
                    );
                } else {
                    info!(
                        _time = end_time.timestamp_micros(),
                        code = %response.status().as_u16(),
                        process_time = time,
                        "Request end"
                    );
                }
            }
            Err(error) => {
                let response_error = error.as_response_error();
                error!(
                    code = %response_error.status_code().as_u16(),
                    error = %response_error,
                    "Request end"
                );
            }
        };
    }
}

fn print_cover() {
    println!("            __                         __   ");
    println!("      _____|  | _____.__. ____   _____/  |_ ");
    println!("     /  ___/  |/ <   |  |/    \\_/ __ \\   __\\");
    println!("     \\___ \\|    < \\___  |   |  \\  ___/|  |  ");
    println!("    /____  >__|_ \\/ ____|___|  /\\___  >__|  ");
    println!("         \\/     \\/\\/         \\/     \\/      \n");
}

pub async fn command(cli: &Cli, skynet: Skynet, skip_cover: bool, disable_csrf: bool) {
    if !skip_cover {
        print_cover();
    }
    let (mut skynet, db, mut redis) = init_skynet(cli, skynet).await;
    if disable_csrf {
        warn!("CSRF protection is disabled, for debugging purpose only");
    }
    if !skynet.config.listen_ssl.get() && !skynet.config.proxy_enable.get() {
        warn!("SSL is not enabled, your traffic is at risk")
    }
    if !cli.persist_session {
        let _: () = redis::cmd("FLUSHDB").query_async(&mut redis).await.unwrap();
    }

    let mut worker: usize = skynet.config.listen_worker.get().try_into().unwrap();
    if worker == 0 {
        worker = num_cpus::get_physical();
    }
    // run server
    skynet.start_time = Utc::now();
    let skynet = Data::new(skynet);
    let cli_data = Data::new(cli.clone());
    let db = Data::new(db);
    let redis = Data::new(redis);
    let stop_handle = Data::new(StopHandle::default());
    let server = HttpServer::new({
        let stop_handle = stop_handle.clone();
        let skynet = skynet.clone();
        move || {
            let mut route = api::new_api(&skynet);
            route = skynet.plugin.parse_route(&skynet, route);

            App::new()
                .service(
                    scope("/api")
                        .configure(api::router(route, disable_csrf))
                        .wrap(get_session_middleware(&skynet, &redis)),
                )
                .service(
                    actix_files::Files::new("/", "./assets")
                        .index_file("index.html")
                        .default_handler(fn_service(|req: ServiceRequest| async {
                            let (req, _) = req.into_parts();
                            let file = NamedFile::open_async("./assets/index.html").await?;
                            let res = file.into_response(&req);
                            Ok(ServiceResponse::new(req, res))
                        })),
                )
                .wrap(middleware::Compress::default())
                .wrap(get_security_header(
                    skynet.config.listen_ssl.get(),
                    skynet.config.header_csp.get().to_owned(),
                ))
                .wrap(TracingLogger::<SpanBuilder>::new())
                .app_data(skynet.clone())
                .app_data(cli_data.clone())
                .app_data(db.clone())
                .app_data(redis.clone())
                .app_data(stop_handle.clone())
        }
    })
    .workers(worker);

    let address = skynet.config.listen_address.get();
    let server = if skynet.config.listen_ssl.get() {
        server
            .bind_rustls_0_22(
                address,
                load_rustls_config(
                    skynet.config.listen_ssl_cert.get(),
                    skynet.config.listen_ssl_key.get(),
                ),
            )
            .unwrap()
            .run()
    } else {
        server.bind(address).unwrap().run()
    };
    stop_handle.register(server.handle());
    *skynet.running.write() = true;
    info!("Listening on {address}");
    server.await.unwrap();
}
