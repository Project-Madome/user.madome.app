use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{convert::Infallible, net::SocketAddr};

use hyper::{
    body::Body,
    http::{Request, Response},
    service::{make_service_fn, service_fn},
};
use inspect::{Inspect, InspectOk};
use sai::{Component, ComponentLifecycle, Injected};

use crate::config::Config;
use crate::model::{Model, Presenter};
use crate::msg::Msg;
use crate::repository::RepositorySet;
use crate::usecase::{create_user, get_user};

#[derive(Component)]
pub struct Resolver {
    #[injected]
    repository: Injected<RepositorySet>,
    // #[injected]
    // command: Injected<CommandSet>,

    // #[injected]
    // config: Injected<Config>,
}

impl Resolver {
    async fn resolve(&self, msg: Msg) -> crate::Result<Model> {
        let repository = Arc::clone(&self.repository);
        // let command = Arc::clone(&self.command);
        // let config = Arc::clone(&self.config);

        let model = match msg {
            Msg::CreateUser(payload) => create_user::execute(payload, repository).await?.into(),

            Msg::GetUser(payload) => get_user::execute(payload, repository).await?.into(),
        };

        Ok(model)
    }
}

#[derive(Component)]
#[lifecycle]
pub struct HttpServer {
    #[injected]
    resolver: Injected<Resolver>,
    /* tx: Option<mpsc::Sender<()>>,
    rx: Option<mpsc::Receiver<()>>, */
    #[injected]
    config: Injected<Config>,
}

async fn handler(
    request: Request<Body>,
    resolver: Arc<Resolver>,
    config: Arc<Config>,
) -> crate::Result<Response<Body>> {
    let response = Response::builder();

    let (msg, response) = Msg::http(request, response, config).await?;

    let model = resolver.resolve(msg).await?;

    let response = model.to_http(response);

    Ok(response)
}

async fn service(
    request: Request<Body>,
    resolver: Arc<Resolver>,
    config: Arc<Config>,
) -> Result<Response<Body>, Infallible> {
    let req_method = request.method().to_owned();
    let req_uri = request.uri().to_string();

    log::info!("HTTP Request {} {}", req_method, req_uri);

    let start = SystemTime::now();

    let response = handler(request, resolver, config).await;

    let end = start
        .elapsed()
        .as_ref()
        .map(Duration::as_millis)
        .unwrap_or(0);

    match response {
        Ok(response) => Ok(response),
        Err(err) => Ok(err.inspect(|e| log::error!("{}", e)).into()),
    }
    .inspect_ok(|res| {
        log::info!(
            "HTTP Response {} {} {} {}",
            req_method,
            req_uri,
            res.status(),
            end
        )
    })
}

#[async_trait::async_trait]
impl ComponentLifecycle for HttpServer {
    async fn start(&mut self) {
        /* let (tx, rx) = mpsc::channel(8);

        self.tx.replace(tx);
        self.rx.replace(rx); */

        let resolver = Arc::clone(&self.resolver);
        let config = Arc::clone(&self.config);

        let port = self.config.port();

        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], port));

            let svc = |resolver: Arc<Resolver>, config: Arc<Config>| async move {
                Ok::<_, Infallible>(service_fn(move |request| {
                    service(request, resolver.clone(), config.clone())
                }))
            };

            let server = hyper::Server::bind(&addr).serve(make_service_fn(move |_| {
                svc(resolver.clone(), config.clone())
            }));

            log::info!("started http server: 0.0.0.0:{}", port);

            if let Err(err) = server.await {
                panic!("{:?}", err);
            }
        });
    }

    async fn stop(&mut self) {}
}

/* pub async fn app(request: Request<Body>) -> crate::Result<Response<Body>> {
    /* let app = tower::service_fn(|request: Request<Body>| async move {
        let msg = Msg::from(&request);

        let model = resolve(msg).await;

        let response = present(model).await;

        response
    }); */

    let msg = Msg::try_from(request).await?;

    let model = resolve(msg).await?;

    let response = model.present()?;

    Ok(response)
} */
