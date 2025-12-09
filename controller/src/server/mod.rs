mod layers;
mod ota;
mod routers;

use embassy_time::Duration;
use layers::PreHandlerLogLayer;
use picoserve::{
    AppRouter, AppWithStateBuilder,
    response::{ErrorWithStatusCode, IntoResponse, Response, StatusCode},
    routing::PathRouter,
};

use crate::{SharedStorage, WEB_TASK_POOL_SIZE, error::Error, panel::Panel};

/// Application state for the web server
///
/// This struct contains all the state needed by the web server,
/// including shared references to the Panel instance and flash storage.
#[derive(Clone)]
pub struct AppState {
    /// Shared reference to the Panel instance
    pub panel: &'static Panel,
    /// Shared reference to the flash storage
    pub storage: SharedStorage,
}

impl picoserve::extract::FromRef<AppState> for &Panel {
    /// Extracts a Panel reference from an AppState
    ///
    /// # Arguments
    /// * `state` - The AppState to extract from
    ///
    /// # Returns
    /// * The Panel reference from the AppState
    fn from_ref(state: &AppState) -> Self {
        state.panel
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
        let router = picoserve::Router::new()
            .nest("/page", routers::page_router())
            .nest("/pages", routers::pages_router())
            .nest("/schedule", routers::schedule_router())
            .nest("/schedules", routers::schedules_router())
            .nest("/clock", routers::clock_router())
            .nest("/reset", routers::delete_all_router())
            .nest("/ota", routers::ota_router())
            .layer(PreHandlerLogLayer);

        #[cfg(feature = "web_interface")]
        let router = router.nest("/", routers::static_router());

        router
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
/// * `id` - Task ID for identifying this web task instance
/// * `stack` - Network stack for TCP/IP communication
/// * `app` - Web application router containing all route handlers
/// * `config` - Web server configuration including timeouts
/// * `state` - Application state shared across all request handlers
#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
    state: AppState,
) -> ! {
    log::info!("Server: Starting webserver listener {id}");
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
