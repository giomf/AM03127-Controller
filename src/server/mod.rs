pub mod dto;
mod layers;
mod routers;

use crate::panel::Panel;
use crate::{WEB_TASK_POOL_SIZE, error::Error};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Duration;
use layers::PreHandlerLogLayer;
use picoserve::response::{ErrorWithStatusCode, Response, StatusCode};
use picoserve::{AppRouter, AppWithStateBuilder, response::IntoResponse, routing::PathRouter};

#[derive(Clone, Copy)]
pub struct SharedPanel(pub &'static Mutex<CriticalSectionRawMutex, Panel<'static>>);

#[derive(Clone)]
pub struct AppState {
    pub shared_panel: SharedPanel,
}

impl picoserve::extract::FromRef<AppState> for SharedPanel {
    fn from_ref(state: &AppState) -> Self {
        state.shared_panel
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppProps;

impl AppWithStateBuilder for AppProps {
    type State = AppState;
    type PathRouter = impl PathRouter<AppState>;

    fn build_app(self) -> picoserve::Router<Self::PathRouter, Self::State> {
        picoserve::Router::new()
            .nest("/", routers::static_router())
            .nest("/page", routers::page_router())
            .nest("/pages", routers::pages_router())
            .nest("/schedule", routers::schedule_router())
            .nest("/schedules", routers::schedules_router())
            .nest("/clock", routers::clock_router())
            .layer(PreHandlerLogLayer)
    }
}

impl ErrorWithStatusCode for Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl IntoResponse for Error {
    async fn write_to<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        self,
        connection: picoserve::response::Connection<'_, R>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        let (status_code, message) = match self {
            Error::Storage(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            Error::Uart(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            Error::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            Error::NotFound(message) => (StatusCode::NOT_FOUND, message),
            Error::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
        };
        let response = Response::new(status_code, message.as_str());
        response_writer.write_response(connection, response).await
    }
}

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
    state: AppState,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::listen_and_serve_with_state(
        id,
        app,
        config,
        stack,
        port,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
        &state,
    )
    .await
}
