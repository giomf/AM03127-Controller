use crate::{uart::Uart, WEB_TASK_POOL_SIZE};
use embassy_time::Duration;
use picoserve::{
    extract::State, routing::{get, PathRouter}, AppBuilder, AppRouter, AppWithStateBuilder
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};


#[derive(Clone, Copy)]
pub struct SharedUart(pub &'static Mutex<CriticalSectionRawMutex, Uart<'static>>);

pub struct AppState {
    pub shared_uart: SharedUart
}

impl picoserve::extract::FromRef<AppState> for SharedUart {
    fn from_ref(state: &AppState) -> Self {
        state.shared_uart
    }
}


pub struct AppProps;

impl AppWithStateBuilder for AppProps {
    type State = AppState;
    type PathRouter = impl PathRouter<AppState>;

    fn build_app(self) -> picoserve::Router<Self::PathRouter, Self::State> {
         picoserve::Router::new()
              .route(
                  "",
                get(
                    |State(SharedUart(shared_uart)): State<SharedUart>| async move {
                        log::info!("");
                    },
                ),
            ) 
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
