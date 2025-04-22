pub mod dto;

use crate::am03127::delete::{DeletePage, DeleteSchedule};
use crate::am03127::page_content::formatting::{Clock as ClockFormat, ColumnStart, Font};
use crate::am03127::realtime_clock::DateTime;
use crate::am03127::schedule::Schedule;
use crate::am03127::{CommandAble, DEFAULT_PANEL_ID};
use crate::storage::NvsStorage;
use crate::{WEB_TASK_POOL_SIZE, am03127::page_content::Page, uart::Uart};
use core::convert::From;
use core::fmt::Write;
use dto::{DateTimeDto, PageDto, ScheduleDto};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Duration;
use heapless::String;
use picoserve::extract::Json;
use picoserve::routing::{get_service, parse_path_segment, post};
use picoserve::{
    AppRouter, AppWithStateBuilder,
    extract::State,
    routing::{PathRouter, get},
};

const JSON_DESERIALIZE_BUFFER_SIZE: usize = 128;

#[derive(Clone, Copy)]
pub struct SharedUart(pub &'static Mutex<CriticalSectionRawMutex, Uart<'static>>);

#[derive(Clone, Copy)]
pub struct SharedStorage(pub &'static Mutex<CriticalSectionRawMutex, NvsStorage>);

#[derive(Clone)]
pub struct AppState {
    pub shared_uart: SharedUart,
    pub shared_storage: SharedStorage,
}

impl picoserve::extract::FromRef<AppState> for SharedUart {
    fn from_ref(state: &AppState) -> Self {
        state.shared_uart
    }
}
impl picoserve::extract::FromRef<AppState> for SharedStorage {
    fn from_ref(state: &AppState) -> Self {
        state.shared_storage
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppProps;

impl AppProps {
    pub fn static_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
        picoserve::Router::new().route(
            "/",
            get_service(picoserve::response::File::html(include_str!("index.html"))),
        )
    }

    pub fn clock_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
        picoserve::Router::new().route(
            "/clock",
            get(
                |State(SharedUart(shared_uart)): State<SharedUart>| async move {
                    log::info!("Display clock");

                    let mut message = String::<64>::new();
                    write!(
                        &mut message,
                        "{}{}{}{}",
                        ClockFormat::Time,
                        Font::Narrow,
                        ColumnStart(41),
                        ClockFormat::Date
                    )
                    .unwrap();

                    let command = Page::default()
                        .message(&message.as_str())
                        .command(DEFAULT_PANEL_ID);
                    shared_uart
                        .lock()
                        .await
                        .write(command.as_bytes())
                        .await
                        .unwrap();
                },
            )
            .post(
                |State(SharedUart(shared_uart)): State<SharedUart>,
                 Json::<DateTimeDto, JSON_DESERIALIZE_BUFFER_SIZE>(date_time_dto)| async move {
                    log::info!("Set clock");

                    let command = DateTime::from(date_time_dto).command(DEFAULT_PANEL_ID);

                    shared_uart
                        .lock()
                        .await
                        .write(command.as_bytes())
                        .await
                        .unwrap();
                },
            ),
        )
    }

    pub fn page_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
        picoserve::Router::new().route(
            ("/page", parse_path_segment::<char>()),
            post(
                |page_id,
                 State(SharedUart(shared_uart)): State<SharedUart>,
                 Json::<PageDto, JSON_DESERIALIZE_BUFFER_SIZE>(page)| async move {
                    log::info!("Setting page {page_id}");

                    let command = Page::from_dto_with_id(page_id, page).command(DEFAULT_PANEL_ID);

                    shared_uart
                        .lock()
                        .await
                        .write(command.as_bytes())
                        .await
                        .unwrap();
                },
            )
            .delete(
                |page_id, State(SharedUart(shared_uart)): State<SharedUart>| async move {
                    log::info!("Delete page {page_id}");
                    let command = DeletePage::default()
                        .page_id(page_id)
                        .command(DEFAULT_PANEL_ID);

                    shared_uart
                        .lock()
                        .await
                        .write(command.as_bytes())
                        .await
                        .unwrap();
                },
            ),
        )
    }

    pub fn schedule_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
        picoserve::Router::new().route(
            ("/schedule", parse_path_segment::<char>()),
            post(
                |schedule_id,
                 State(SharedUart(shared_uart)): State<SharedUart>,
                 Json::<ScheduleDto, JSON_DESERIALIZE_BUFFER_SIZE>(schedule)| async move {
                    log::info!("Setting schedule {schedule_id}");
                    let command =
                        Schedule::from_dto_with_id(schedule, schedule_id).command(DEFAULT_PANEL_ID);

                    shared_uart
                        .lock()
                        .await
                        .write(command.as_bytes())
                        .await
                        .unwrap();
                },
            )
            .delete(
                |schedule_id, State(SharedUart(shared_uart)): State<SharedUart>| async move {
                    log::info!("Deleting schedule {schedule_id}");
                    let command = DeleteSchedule::new(schedule_id).command(DEFAULT_PANEL_ID);

                    shared_uart
                        .lock()
                        .await
                        .write(command.as_bytes())
                        .await
                        .unwrap();
                },
            ),
        )
    }
}

impl AppWithStateBuilder for AppProps {
    type State = AppState;
    type PathRouter = impl PathRouter<AppState>;

    fn build_app(self) -> picoserve::Router<Self::PathRouter, Self::State> {
        picoserve::Router::new()
            .nest("/", Self::static_router())
            .nest("/", Self::clock_router())
            .nest("/", Self::page_router())
            .nest("/", Self::schedule_router())
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
