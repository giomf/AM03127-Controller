pub mod dto;
mod router;

use crate::WEB_TASK_POOL_SIZE;
use crate::panel::Panel;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Duration;
use picoserve::{AppRouter, AppWithStateBuilder, routing::PathRouter};

const JSON_DESERIALIZE_BUFFER_SIZE: usize = 128;

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
            .nest("/", router::static_router())
            .nest("/page", router::page_router())
            .nest("/pages", router::pages_router())
            .nest("/schedule", router::schedule_router())
            .nest("/clock", router::clock_router())
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
