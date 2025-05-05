use super::{AppState, SharedPanel, dto};
use crate::am03127::page_content::Page;
use crate::am03127::realtime_clock::DateTime;
use crate::am03127::schedule::Schedule;
use crate::error::Error;
use core::convert::From;
use dto::{DateTimeDto, PageDto, ScheduleDto};
use picoserve::extract::Json;
use picoserve::routing::{get_service, parse_path_segment};
use picoserve::{
    extract::State,
    routing::{PathRouter, get},
};

/// Logger name for router-related log messages
const LOGGER_NAME: &str = "Router";
/// Buffer size for JSON deserialization
const JSON_DESERIALIZE_BUFFER_SIZE: usize = 128;

/// Creates a router for static content
///
/// # Returns
/// * A router that serves static content
pub fn static_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
    picoserve::Router::new().route(
        "",
        get_service(picoserve::response::File::html(include_str!("index.html"))),
    )
}

/// Creates a router for clock-related endpoints
///
/// # Returns
/// * A router that handles clock-related requests
pub fn clock_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
    picoserve::Router::new().route(
        "",
        get(
            |State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                log::info!("{LOGGER_NAME}: Display clock");

                let mut panel = shared_panel.lock().await;
                match panel.display_clock('A').await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
        )
        .post(
            |State(SharedPanel(shared_panel)): State<SharedPanel>,
             Json::<DateTimeDto, JSON_DESERIALIZE_BUFFER_SIZE>(date_time_dto)| async move {
                log::info!("{LOGGER_NAME}: Set clock");
                let date_time = DateTime::from(date_time_dto);

                let mut panel = shared_panel.lock().await;
                match panel.set_clock(date_time).await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
        ),
    )
}

/// Creates a router for page-related endpoints
///
/// # Returns
/// * A router that handles requests for individual pages
pub fn page_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
    picoserve::Router::new().route(
        ("", parse_path_segment::<char>()),
        get(
            |page_id: char, State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                log::info!("{LOGGER_NAME}: Getting page \"{page_id}\"");
                if !is_id_valid(page_id) {
                    return Err(Error::BadRequest("Page ID not valid".into()));
                }

                let mut panel = shared_panel.lock().await;
                match panel.get_page(page_id).await {
                    Ok(Some(page)) => Ok(Json(page)),
                    Ok(None) => Err(Error::NotFound("Page not found".into())),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
        )
        .post(
            |page_id: char,
             State(SharedPanel(shared_panel)): State<SharedPanel>,
             Json::<PageDto, JSON_DESERIALIZE_BUFFER_SIZE>(page_dto)| async move {
                log::info!("{LOGGER_NAME}: Setting page \"{page_id}\"");
                if !is_id_valid(page_id) {
                    return Err(Error::BadRequest("Page ID not valid".into()));
                }
                log::debug!("{LOGGER_NAME}: {:?}", page_dto);

                let page = Page::from_dto_with_id(page_id, page_dto);
                let mut panel = shared_panel.lock().await;
                match panel.set_page(page_id, page).await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
        )
        .delete(
            |page_id: char, State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                if !is_id_valid(page_id) {
                    return Err(Error::BadRequest("Page ID not valid".into()));
                }
                log::info!("{LOGGER_NAME}: Delete page \"{page_id}\"");

                let mut panel = shared_panel.lock().await;
                match panel.delete_page(page_id).await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
        ),
    )
}

/// Creates a router for the pages collection endpoint
///
/// # Returns
/// * A router that handles requests for all pages
pub fn pages_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
    picoserve::Router::new().route(
        "",
        get(
            |State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                let mut panel = shared_panel.lock().await;
                match panel.get_pages().await {
                    Ok(pages) => Ok(Json(pages)),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
        ),
    )
}

/// Creates a router for schedule-related endpoints
///
/// # Returns
/// * A router that handles requests for individual schedules
pub fn schedule_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
    picoserve::Router::new().route(
        ("", parse_path_segment::<char>()),
        get(
            |schedule_id: char, State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                log::info!("{LOGGER_NAME}: Getting page \"{schedule_id}\"");
                if !is_id_valid(schedule_id) {
                    return Err(Error::BadRequest("Schedule ID not valid".into()));
                }

                let mut panel = shared_panel.lock().await;
                match panel.get_schedule(schedule_id).await {
                    Ok(Some(schedule)) => Ok(Json(schedule)),
                    Ok(None) => Err(Error::NotFound("Schedule not found".into())),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
        )
        .post(
            |schedule_id: char,
             State(SharedPanel(shared_panel)): State<SharedPanel>,
             Json::<ScheduleDto, JSON_DESERIALIZE_BUFFER_SIZE>(schedule)| async move {
                log::info!("{LOGGER_NAME}: Setting schedule {schedule_id}");
                if !is_id_valid(schedule_id) {
                    return Err(Error::BadRequest("Schedule ID not valid".into()));
                }
                let schedule = Schedule::from_dto_with_id(schedule, schedule_id);

                let mut panel = shared_panel.lock().await;
                match panel.set_schedule(schedule_id, schedule).await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
        )
        .delete(
            |schedule_id: char, State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                log::info!("{LOGGER_NAME}: Deleting schedule {schedule_id}");
                if !is_id_valid(schedule_id) {
                    return Err(Error::BadRequest("Schedule ID not valid".into()));
                }

                let mut panel = shared_panel.lock().await;
                match panel.delete_schedule(schedule_id).await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
        ),
    )
}

/// Creates a router for the schedules collection endpoint
///
/// # Returns
/// * A router that handles requests for all schedules
pub fn schedules_router() -> picoserve::Router<impl PathRouter<AppState>, AppState> {
    picoserve::Router::new().route(
        "",
        get(
            |State(SharedPanel(shared_panel)): State<SharedPanel>| async move {
                let mut panel = shared_panel.lock().await;
                match panel.get_schedules().await {
                    Ok(schedules) => Ok(Json(schedules)),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
        ),
    )
}

/// Checks if an ID is valid (A-Z)
///
/// # Arguments
/// * `id` - The ID to check
///
/// # Returns
/// * `true` if the ID is valid (A-Z)
/// * `false` otherwise
fn is_id_valid(id: char) -> bool {
    id >= 'A' && id <= 'Z'
}
