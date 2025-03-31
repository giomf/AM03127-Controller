use crate::WEB_TASK_POOL_SIZE;
use embassy_time::Duration;
use picoserve::{
    AppBuilder, AppRouter,
    routing::get,
};

#[derive(serde::Deserialize)]
struct QueryParams {
    a: i32,
    b: heapless::String<32>,
}


pub struct AppProps;
impl AppBuilder for AppProps {
    type PathRouter = impl picoserve::routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::new()
            .route(
                "/",
                  get(|picoserve::extract::Query(QueryParams { a, b })| {
                    picoserve::response::DebugValue((("a", a), ("b", b)))
                })
            )
    }
}

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::listen_and_serve(
        id,
        app,
        config,
        stack,
        port,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
    )
    .await
}
