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

/// Shared reference to the Panel instance
///
/// This wrapper allows sharing the Panel instance between multiple tasks
/// while ensuring thread-safe access through a mutex.
#[derive(Clone, Copy)]
pub struct SharedPanel(pub &'static Mutex<CriticalSectionRawMutex, Panel<'static>>);

/// Application state for the web server
///
/// This struct contains all the state needed by the web server,
/// including a shared reference to the Panel instance.
#[derive(Clone)]
pub struct AppState {
    /// Shared reference to the Panel instance
    pub shared_panel: SharedPanel,
}

impl picoserve::extract::FromRef<AppState> for SharedPanel {
    /// Extracts a SharedPanel from an AppState
    ///
    /// # Arguments
    /// * `state` - The AppState to extract from
    ///
    /// # Returns
    /// * The SharedPanel from the AppState
    fn from_ref(state: &AppState) -> Self {
        state.shared_panel
    }
}

/// Properties for building the web application
#[derive(Debug, Clone, Default)]
pub struct AppProps;

impl AppWithStateBuilder for AppProps {
    type State = AppState;
    type PathRouter = impl PathRouter<AppState>;

    /// Builds the web application router
    ///
    /// # Returns
    /// * A router configured with all the application's routes
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
    /// Returns the HTTP status code for an error
    ///
    /// # Returns
    /// * The appropriate HTTP status code for the error
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl IntoResponse for Error {
    /// Converts an Error to an HTTP response
    ///
    /// # Arguments
    /// * `connection` - The HTTP connection
    /// * `response_writer` - Writer for the HTTP response
    ///
    /// # Returns
    /// * `Ok(ResponseSent)` if the response was sent successfully
    /// * `Err(W::Error)` if sending the response failed
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

/// Web server task
///
/// This task runs the web server that handles HTTP requests.
///
/// # Arguments
/// * `id` - Task ID
/// * `stack` - Network stack
/// * `app` - Web application router
/// * `config` - Web server configuration
/// * `state` - Application state
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
