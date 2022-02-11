use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{convert::Infallible, net::SocketAddr};

use hyper::Server;
use hyper::{
    body::Body,
    http::{Request, Response},
    service::{make_service_fn, service_fn},
};
use inspect::{Inspect, InspectOk};
use sai::{Component, ComponentLifecycle, Injected};
use tokio::sync::oneshot;

use crate::config::Config;
use crate::model::{Model, Presenter};
use crate::msg::Msg;
use crate::repository::RepositorySet;
use crate::usecase::{
    create_like, create_user, delete_like, get_likes, get_likes_from_book_tags, get_user,
};

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

            Msg::CreateLike(payload) => create_like::execute(payload, repository).await?.into(),

            Msg::GetLikes(payload) => get_likes::execute(payload, repository).await?.into(),

            Msg::DeleteLike(payload) => delete_like::execute(payload, repository).await?.into(),

            Msg::GetLikesFromBookTags(payload) => {
                get_likes_from_book_tags::execute(payload, repository)
                    .await?
                    .into()
            }
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

    stop_sender: Option<oneshot::Sender<()>>,

    stopped_reciever: Option<oneshot::Receiver<()>>,
}

async fn handler(
    request: Request<Body>,
    resolver: Arc<Resolver>,
    auth_url: String,
) -> crate::Result<Response<Body>> {
    let response = Response::builder();

    let (msg, response) = Msg::http(request, response, auth_url).await?;

    let model = resolver.resolve(msg).await?;

    let response = model.to_http(response);

    Ok(response)
}

async fn service(
    request: Request<Body>,
    resolver: Arc<Resolver>,
    auth_url: String,
) -> Result<Response<Body>, Infallible> {
    let req_method = request.method().to_owned();
    let req_uri = request.uri().to_string();

    log::info!("--> {} {}", req_method, req_uri);

    let start = SystemTime::now();

    let response = handler(request, resolver, auth_url).await;

    let end = start
        .elapsed()
        .as_ref()
        .map(Duration::as_micros)
        .unwrap_or(0);

    match response {
        Ok(response) => Ok(response),
        Err(err) => Ok(err.inspect(|e| log::error!("{}", e)).into()),
    }
    .inspect_ok(|res| {
        log::info!(
            "<-- {} {} {} {}ms",
            req_method,
            req_uri,
            res.status(),
            end as f64 / 1000.0
        )
    })
}

#[async_trait::async_trait]
impl ComponentLifecycle for HttpServer {
    async fn start(&mut self) {
        /* let (tx, rx) = mpsc::channel(8);

        self.tx.replace(tx);
        self.rx.replace(rx); */

        let (stop_tx, stop_rx) = oneshot::channel();
        let (stopped_tx, stopped_rx) = oneshot::channel();

        self.stop_sender.replace(stop_tx);
        self.stopped_reciever.replace(stopped_rx);

        let resolver = Arc::clone(&self.resolver);
        let madome_auth_url = self.config.madome_auth_url().to_owned();

        let port = self.config.port();

        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], port));

            let svc = |resolver: Arc<Resolver>, madome_auth_url: String| async move {
                Ok::<_, Infallible>(service_fn(move |request| {
                    service(request, resolver.clone(), madome_auth_url.clone())
                }))
            };

            let server = Server::bind(&addr).serve(make_service_fn(move |_| {
                svc(resolver.clone(), madome_auth_url.clone())
            }));

            let server = Server::with_graceful_shutdown(server, async {
                stop_rx.await.unwrap();
            });

            log::info!("started http server: 0.0.0.0:{}", port);

            if let Err(err) = server.await {
                log::error!("{:?}", err);
            }

            stopped_tx.send(()).unwrap();
        });
    }

    async fn stop(&mut self) {
        let stop_tx = self.stop_sender.take().unwrap();

        stop_tx.send(()).unwrap();

        let stopped_rx = self.stopped_reciever.take().unwrap();

        stopped_rx.await.unwrap();
    }
}
