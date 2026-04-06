mod layers;
mod ota;
mod routers;

use layers::PreHandlerLogLayer;
use picoserve::{
    AppRouter, AppWithStateBuilder,
    response::{ErrorWithStatusCode, IntoResponse, Response, StatusCode},
    routing::{PathRouter, parse_path_segment},
};

use crate::{SharedStorage, WEB_TASK_POOL_SIZE, error::Error, panel::Panel};

/// Shared reference to the Panel instance
// pub type SharedPanel = &'static Mutex<CriticalSectionRawMutex, Panel>;

/// Application state for the web server
///
/// This struct contains all the state needed by the web server,
/// including a shared reference to the Panel instance.
pub struct AppState {
    /// Shared reference to the Panel instance
    pub panel: &'static Panel,
    /// Shared reference to the Panel instance
    pub storage: SharedStorage,
}

impl picoserve::extract::FromRef<AppState> for &Panel {
    /// Extracts a SharedPanel from an AppState
    ///
    /// # Arguments
    /// * `state` - The AppState to extract from
    ///
    /// # Returns
    /// * The SharedPanel from the AppState
    fn from_ref(state: &AppState) -> Self {
        state.panel
    }
}

/// Properties for building the web application
pub struct ServerProperties;

impl AppWithStateBuilder for ServerProperties {
    type State = AppState;
    type PathRouter = impl PathRouter<AppState>;

    /// Builds the web application router
    ///
    /// # Returns
    /// * A router configured with all the application's routes
    fn build_app(self) -> picoserve::Router<Self::PathRouter, Self::State> {
        let router = picoserve::Router::new()
            .route(
                ("/page", parse_path_segment::<char>()),
                routers::page_router(),
            )
            .route("/pages", routers::pages_router())
            .route(
                ("/schedule", parse_path_segment::<char>()),
                routers::schedule_router(),
            )
            .route("/schedules", routers::schedules_router())
            .route("/clock", routers::clock_router())
            .route("/reset", routers::delete_all_router())
            .route("/ota", routers::ota_router())
            .route("/status", routers::status_router())
            .layer(PreHandlerLogLayer);

        #[cfg(feature = "web_interface")]
        let router = router.route("/", routers::static_router());

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
/// * `id` - Task ID
/// * `stack` - Network stack
/// * `app` - Web application router
/// * `config` - Web server configuration
/// * `state` - Application state
#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    server_properties: &'static AppRouter<ServerProperties>,
    config: &'static picoserve::Config,
    state: AppState,
) -> ! {
    log::info!("Server: Starting webserver listener {id}");
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::Server::new(
        &server_properties.shared().with_state(&state),
        config,
        &mut http_buffer,
    )
    .listen_and_serve(id, stack, port, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
    .await
    .into_never()
}
