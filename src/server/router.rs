use super::{AppState, SharedPanel, dto};
use crate::JSON_DESERIALIZE_BUFFER_SIZE;
use crate::am03127::page_content::Page;
use crate::am03127::realtime_clock::DateTime;
use crate::am03127::schedule::Schedule;
use core::convert::From;
use dto::{DateTimeDto, PageDto, ScheduleDto};
use picoserve::extract::Json;
use picoserve::response::StatusCode;
use picoserve::routing::{get_service, parse_path_segment};
use picoserve::{
    extract::State,
    routing::{PathRouter, get},
};

const LOGGER_NAME: &str = "Router";

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
                log::info!("{LOGGER_NAME}: Display clock");

                let mut panel = shared_panel.lock().await;
                panel
                    .display_clock('A')
                    .await
                    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))
            },
        )
        .post(
            |State(SharedPanel(shared_panel)): State<SharedPanel>,
             Json::<DateTimeDto, JSON_DESERIALIZE_BUFFER_SIZE>(date_time_dto)| async move {
                log::info!("{LOGGER_NAME}: Set clock");
                let date_time = DateTime::from(date_time_dto);

                let mut panel = shared_panel.lock().await;
                panel
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
                log::info!("{LOGGER_NAME}: Getting page \"{page_id}\"");
                if !is_id_valid(page_id) {
                    return Err((StatusCode::BAD_REQUEST, "Page id not valid"));
                }

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
                log::info!("{LOGGER_NAME}: Setting page \"{page_id}\"");
                if !is_id_valid(page_id) {
                    return Err((StatusCode::BAD_REQUEST, "Page id not valid"));
                }
                log::debug!("{LOGGER_NAME}: {:?}", page_dto);

                let page = Page::from_dto_with_id(page_id, page_dto);
                let mut panel = shared_panel.lock().await;
                panel
                    .set_page(page_id, page)
                    .await
                    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))
            },
        )
        .delete(
            |page_id: char, State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                if !is_id_valid(page_id) {
                    return Err((StatusCode::BAD_REQUEST, "Page id not valid"));
                }
                log::info!("{LOGGER_NAME}: Delete page \"{page_id}\"");

                let mut panel = shared_panel.lock().await;
                panel
                    .delete_page(page_id)
                    .await
                    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete page"))
            },
        ),
    )
}

pub fn pages_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
    picoserve::Router::new().route(
        "",
        get(
            |State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                let mut panel = shared_panel.lock().await;
                match panel.get_pages().await {
                    Ok(pages) => Ok(Json(pages)),
                    Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get pages")),
                }
            },
        ),
    )
}

pub fn schedule_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
    picoserve::Router::new().route(
        ("", parse_path_segment::<char>()),
        get(
            |schedule_id: char, State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                log::info!("{LOGGER_NAME}: Getting page \"{schedule_id}\"");
                if !is_id_valid(schedule_id) {
                    return Err((StatusCode::BAD_REQUEST, "Schedule id not valid"));
                }

                let mut panel = shared_panel.lock().await;
                if let Ok(schedule) = panel.get_schedule(schedule_id).await {
                    match schedule {
                        Some(schedule) => Ok(Json(schedule)),
                        None => Err((StatusCode::NOT_FOUND, "Schedule not found")),
                    }
                } else {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get schedule"))
                }
            },
        )
        .post(
            |schedule_id: char,
             State(SharedPanel(shared_panel)): State<SharedPanel>,
             Json::<ScheduleDto, JSON_DESERIALIZE_BUFFER_SIZE>(schedule)| async move {
                log::info!("{LOGGER_NAME}: Setting schedule {schedule_id}");
                if !is_id_valid(schedule_id) {
                    return Err((StatusCode::BAD_REQUEST, "Schedule id not valid"));
                }
                let schedule = Schedule::from_dto_with_id(schedule, schedule_id);

                let mut panel = shared_panel.lock().await;
                panel
                    .set_schedule(schedule_id, schedule)
                    .await
                    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))
            },
        )
        .delete(
            |schedule_id: char, State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                log::info!("{LOGGER_NAME}: Deleting schedule {schedule_id}");
                if !is_id_valid(schedule_id) {
                    return Err((StatusCode::BAD_REQUEST, "Schedule id not valid"));
                }

                let mut panel = shared_panel.lock().await;
                panel
                    .delete_schedule(schedule_id)
                    .await
                    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))
            },
        ),
    )
}

pub fn schedules_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
    picoserve::Router::new().route(
        "",
        get(
            |State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                let mut panel = shared_panel.lock().await;
                match panel.get_schedules().await {
                    Ok(schedules) => Ok(Json(schedules)),
                    Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get schedules")),
                }
            },
        ),
    )
}
fn is_id_valid(id: char) -> bool {
    id > 'A' && id < 'Z'
}
