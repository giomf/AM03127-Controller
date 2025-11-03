use picoserve::{io::Read, response::ResponseWriter};

/// Logger name for HTTP handler-related log messages
const LOGGER_NAME: &str = "Handler";

/// Wrapper for ResponseWriter that logs response status codes
///
/// This struct wraps a ResponseWriter and logs information about
/// the HTTP response status code before writing the response.
struct LogResponseWriter<W> {
    /// The wrapped response writer
    response_writer: W,
}

impl<W: ResponseWriter> ResponseWriter for LogResponseWriter<W> {
    type Error = W::Error;

    /// Writes an HTTP response and logs its status code
    ///
    /// # Arguments
    /// * `connection` - The HTTP connection
    /// * `response` - The HTTP response to write
    ///
    /// # Returns
    /// * `Ok(ResponseSent)` if the response was sent successfully
    /// * `Err(Self::Error)` if sending the response failed
    async fn write_response<
        R: Read<Error = Self::Error>,
        H: picoserve::response::HeadersIter,
        B: picoserve::response::Body,
    >(
        self,
        connection: picoserve::response::Connection<'_, R>,
        response: picoserve::response::Response<H, B>,
    ) -> Result<picoserve::ResponseSent, Self::Error> {
        let status_code = response.status_code();
        if status_code.is_success() {
            log::info!("{LOGGER_NAME}: Returning success {status_code}!");
        } else if status_code.is_client_error() {
            log::warn!("{LOGGER_NAME}: Returning client error {status_code}!");
        } else if status_code.is_server_error() {
            log::error!("{LOGGER_NAME}: Returning server error {status_code}!");
        }

        self.response_writer
            .write_response(connection, response)
            .await
    }
}

/// Middleware layer that logs HTTP requests and responses
///
/// This layer logs information about incoming HTTP requests and their responses.
pub struct PreHandlerLogLayer;

impl<State, PathParameters> picoserve::routing::Layer<State, PathParameters>
    for PreHandlerLogLayer
{
    type NextState = State;
    type NextPathParameters = PathParameters;

    /// Processes an HTTP request, logs information about it, and passes it to the next layer
    ///
    /// # Arguments
    /// * `next` - The next layer in the middleware chain
    /// * `state` - The application state
    /// * `path_parameters` - Parameters extracted from the URL path
    /// * `request_parts` - Parts of the HTTP request
    /// * `response_writer` - Writer for the HTTP response
    ///
    /// # Returns
    /// * `Ok(ResponseSent)` if the response was sent successfully
    /// * `Err(W::Error)` if processing the request or sending the response failed
    async fn call_layer<
        'a,
        R: Read + 'a,
        NextLayer: picoserve::routing::Next<'a, R, Self::NextState, Self::NextPathParameters>,
        W: ResponseWriter<Error = R::Error>,
    >(
        &self,
        next: NextLayer,
        state: &State,
        path_parameters: PathParameters,
        request_parts: picoserve::request::RequestParts<'_>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        let method = request_parts.method();
        let path = request_parts.path();
        log::info!("{LOGGER_NAME}: {method} request to {path}");
        next.run(
            state,
            path_parameters,
            LogResponseWriter { response_writer },
        )
        .await
    }
}
