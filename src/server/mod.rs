pub mod dto;

use crate::am03127::realtime_clock::DateTime;
use crate::am03127::schedule::Schedule;
use crate::panel::Panel;
use crate::{WEB_TASK_POOL_SIZE, am03127::page_content::Page};
use core::convert::From;
use dto::{DateTimeDto, PageDto, ScheduleDto};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Duration;
use picoserve::extract::Json;
use picoserve::response::StatusCode;
use picoserve::routing::{get_service, parse_path_segment};
use picoserve::{
    AppRouter, AppWithStateBuilder,
    extract::State,
    routing::{PathRouter, get},
};

const JSON_DESERIALIZE_BUFFER_SIZE: usize = 128;
const LOGGER_NAME: &str = "HTTP Server";

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

impl AppProps {
    pub fn static_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
        picoserve::Router::new().route(
            "",
            get_service(picoserve::response::File::html(include_str!("index.html"))),
        )
    }

    pub fn clock_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
        picoserve::Router::new().route(
            "",
            get(
                |State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                    log::info!("Display clock");

                    shared_panel
                        .lock()
                        .await
                        .display_clock('A')
                        .await
                        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))
                },
            )
            .post(
                |State(SharedPanel(shared_panel)): State<SharedPanel>,
                 Json::<DateTimeDto, JSON_DESERIALIZE_BUFFER_SIZE>(date_time_dto)| async move {
                    log::info!("Set clock");
                    let date_time = DateTime::from(date_time_dto);

                    shared_panel
                        .lock()
                        .await
                        .set_clock(date_time)
                        .await
                        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))
                },
            ),
        )
    }

    pub fn page_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
        picoserve::Router::new().route(
            ("", parse_path_segment::<char>()),
            get(
                |page_id: char, State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                    let page_id = page_id.to_ascii_uppercase();
                    log::info!("{LOGGER_NAME}: Getting page \"{page_id}\"");

                    let mut panel = shared_panel.lock().await;
                    if let Ok(page) = panel.get_page(page_id).await {
                        match page {
                            Some(page) => Ok(Json(page)),
                            None => Err((StatusCode::NOT_FOUND, "Page not found")),
                        }
                    } else {
                        Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get Page"))
                    }
                },
            )
            .post(
                |page_id: char,
                 State(SharedPanel(shared_panel)): State<SharedPanel>,
                 Json::<PageDto, JSON_DESERIALIZE_BUFFER_SIZE>(page_dto)| async move {
                    let page_id = page_id.to_ascii_uppercase();
                    log::info!("{LOGGER_NAME}: Setting page \"{page_id}\"");
                    log::debug!("{LOGGER_NAME}: {:?}", page_dto);

                    let page = Page::from_dto_with_id(page_id, page_dto);
                    shared_panel
                        .lock()
                        .await
                        .set_page(page_id, page)
                        .await
                        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))
                },
            )
            .delete(
                |page_id: char, State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                    let page_id = page_id.to_ascii_uppercase();
                    log::info!("{LOGGER_NAME}: Delete page \"{page_id}\"");

                    shared_panel
                        .lock()
                        .await
                        .delete_page(page_id)
                        .await
                        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))
                },
            ),
        )
    }

    pub fn schedule_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
        picoserve::Router::new().route(
            ("", parse_path_segment::<char>()),
            get(
                |page_id: char, State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                    let schedule_id = page_id.to_ascii_uppercase();
                    log::info!("{LOGGER_NAME}: Getting page \"{schedule_id}\"");

                    let mut panel = shared_panel.lock().await;
                    if let Ok(schedule) = panel.get_schedule(schedule_id).await {
                        match schedule {
                            Some(schedule) => Ok(Json(schedule)),
                            None => Err((StatusCode::NOT_FOUND, "Schedule not found")),
                        }
                    }else {
                        Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get schedule"))
                    }

                },
            ).
            post(
                |schedule_id: char,
                 State(SharedPanel(shared_panel)): State<SharedPanel>,
                 Json::<ScheduleDto, JSON_DESERIALIZE_BUFFER_SIZE>(schedule)| async move {
                    let schedule_id = schedule_id.to_ascii_uppercase();
                    log::info!("Setting schedule {schedule_id}");
                    let schedule = Schedule::from_dto_with_id(schedule, schedule_id);

                    shared_panel
                        .lock()
                        .await
                        .set_schedule(schedule_id, schedule).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))
                },
            )
            .delete(
                |schedule_id: char, State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                    let schedule_id = schedule_id.to_ascii_uppercase();
                    log::info!("Deleting schedule {schedule_id}");

                    shared_panel.lock().await.delete_schedule(schedule_id).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))
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
            .nest("/page", Self::page_router())
            .nest("/schedule", Self::schedule_router())
            .nest("/clock", Self::clock_router())
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
