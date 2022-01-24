use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};

use hyper::{
    body::Body,
    http::{Request, Response},
    service::{make_service_fn, service_fn},
};
use sai::{Component, ComponentLifecycle, Injected};

use crate::config::Config;
use crate::model::Model;
use crate::msg::Msg;
use crate::repository::RepositorySet;
use crate::usecase::{create_user, get_user};
use crate::utils::r#async::AsyncTryFrom;

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

async fn handler(request: Request<Body>, resolver: Arc<Resolver>) -> crate::Result<Response<Body>> {
    let msg = Msg::async_try_from(request).await?;

    let model = resolver.resolve(msg).await?;

    let response = model.into();

    Ok(response)
}

async fn service(
    request: Request<Body>,
    resolver: Arc<Resolver>,
) -> Result<Response<Body>, Infallible> {
    let response = handler(request, resolver).await;

    match response {
        Ok(response) => Ok(response),
        // TODO: 에러 핸들링
        Err(error) => {
            log::error!("{}", error);
            Ok(error.into())
        }
    }
}

#[async_trait::async_trait]
impl ComponentLifecycle for HttpServer {
    async fn start(&mut self) {
        /* let (tx, rx) = mpsc::channel(8);

        self.tx.replace(tx);
        self.rx.replace(rx); */

        let resolver = Arc::clone(&self.resolver);

        let port = self.config.port();

        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], port));

            let svc = |resolver: Arc<Resolver>| async move {
                Ok::<_, Infallible>(service_fn(move |request| {
                    service(request, Arc::clone(&resolver))
                }))
            };

            let server = hyper::Server::bind(&addr)
                .serve(make_service_fn(move |_| svc(Arc::clone(&resolver))));

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
