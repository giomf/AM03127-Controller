use picoserve::{io::Read, response::ResponseWriter};

struct LogResponseWriter<W> {
    response_writer: W,
}

const LOGGER_NAME: &str = "Handler";

impl<W: ResponseWriter> ResponseWriter for LogResponseWriter<W> {
    type Error = W::Error;

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

pub struct PreHandlerLogLayer;

impl<State, PathParameters> picoserve::routing::Layer<State, PathParameters>
    for PreHandlerLogLayer
{
    type NextState = State;
    type NextPathParameters = PathParameters;

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
