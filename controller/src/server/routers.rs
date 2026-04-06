extern crate alloc;
use alloc::vec::Vec;

use am03127::{page::Page, realtime_clock::DateTime, schedule::Schedule};
#[cfg(feature = "web_interface")]
use picoserve::routing::{MethodHandler, get_service};
use picoserve::{
    extract::{Json, State},
    routing::{get, post, put_service},
};
use serde::Serialize;

use super::AppState;
use crate::{error::Error, panel::Panel, server::ota::OverTheAirUpdate};

/// Logger name for router-related log messages
const LOGGER_NAME: &str = "Router";

#[derive(Serialize)]
struct BuildInfo {
    version: &'static str,
    build_time: &'static str,
    build_date: &'static str,
}

/// Creates a router for the /status endpoint
///
/// # Returns
/// * A router that returns build version, time and date as JSON
pub fn status_router() -> impl MethodHandler<AppState> {
    get(|| async move {
        Json(BuildInfo {
            version: env!("BUILD_VERSION"),
            build_time: env!("BUILD_TIME"),
            build_date: env!("BUILD_DATE"),
        })
    })
}

/// Creates a router for static content
///
/// # Returns
/// * A router that serves static content
#[cfg(feature = "web_interface")]
pub fn static_router() -> impl MethodHandler<AppState> {
    get_service(picoserve::response::File::html(include_str!("index.html")))
}
/// Creates a router for clock-related endpoints
///
/// # Returns
/// * A router that handles clock-related requests
pub fn clock_router() -> impl MethodHandler<AppState> {
    post(
        |State(panel): State<&'static Panel>, Json::<DateTime>(date_time)| async move {
            log::info!("{LOGGER_NAME}: Set clock");

            match panel.set_clock(&date_time).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    log::error!("{LOGGER_NAME}: {err}");
                    Err(err)
                }
            }
        },
    )
}

/// Creates a router for page-related endpoints
///
/// # Returns
/// * A router that handles requests for individual pages
pub fn page_router() -> impl MethodHandler<AppState, (char,)> {
    get(
        |page_id: char, State(panel): State<&'static Panel>| async move {
            log::info!("{LOGGER_NAME}: Getting page \"{page_id}\"");
            if !is_page_id_valid(page_id) {
                return Err(Error::BadRequest("Page ID not valid".try_into().unwrap()));
            }

            match panel.get_page(page_id).await {
                Ok(Some(page)) => Ok(Json(page)),
                Ok(None) => Err(Error::NotFound("Page not found".try_into().unwrap())),
                Err(err) => {
                    log::error!("{LOGGER_NAME}: {err}");
                    Err(err)
                }
            }
        },
    )
    .post(
        |page_id: char, State(panel): State<&'static Panel>, Json::<Page>(page)| async move {
            log::info!("{LOGGER_NAME}: Setting page \"{page_id}\"");
            if !is_page_id_valid(page_id) {
                return Err(Error::BadRequest("Page ID not valid".try_into().unwrap()));
            }
            log::debug!("{LOGGER_NAME}: {:?}", page);

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
        |page_id: char, State(panel): State<&'static Panel>| async move {
            if !is_page_id_valid(page_id) {
                return Err(Error::BadRequest("Page ID not valid".try_into().unwrap()));
            }
            log::info!("{LOGGER_NAME}: Delete page \"{page_id}\"");

            match panel.delete_page(page_id).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    log::error!("{LOGGER_NAME}: {err}");
                    Err(err)
                }
            }
        },
    )
}

/// Creates a router for the pages collection endpoint
///
/// # Returns
/// * A router that handles requests for all pages
pub fn pages_router() -> impl MethodHandler<AppState> {
    get(|State(panel): State<&'static Panel>| async move {
        match panel.get_pages().await {
            Ok(pages) => Ok(Json(pages)),
            Err(err) => {
                log::error!("{LOGGER_NAME}: {err}");
                Err(err)
            }
        }
    })
    .post(
        |State(panel): State<&'static Panel>, Json::<Vec<Page>>(pages)| async move {
            for page in pages {
                if let Err(err) = panel.set_page(page.id, page).await {
                    log::error!("{LOGGER_NAME}: {err}");
                    return Err(err);
                }
            }
            Ok(())
        },
    )
}

//        ("", parse_path_segment::<char>()),
/// Creates a router for schedule-related endpoints
///
/// # Returns
/// * A router that handles requests for individual schedules
pub fn schedule_router() -> impl MethodHandler<AppState, (char,)> {
    get(
            |schedule_id: char, State(panel): State<&'static Panel>| async move {
                log::info!("{LOGGER_NAME}: Getting page \"{schedule_id}\"");
                if !is_page_id_valid(schedule_id) {
                    return Err(Error::BadRequest(
                        "Schedule ID not valid".try_into().unwrap(),
                    ));
                }

                match panel.get_schedule(schedule_id).await {
                    Ok(Some(schedule)) => Ok(Json(schedule)),
                    Ok(None) => Err(Error::NotFound("Schedule not found".try_into().unwrap())),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
        )
        .post(
            |schedule_id: char,
             State(panel): State<&'static Panel>,
             Json::<Schedule>(schedule)| async move {
                log::info!("{LOGGER_NAME}: Setting schedule {schedule_id}");
                if !is_schedule_id_valid(schedule_id) {
                    return Err(Error::BadRequest(
                        "Schedule ID not valid".try_into().unwrap(),
                    ));
                }

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
            |schedule_id: char, State(panel): State<&'static Panel>| async move {
                log::info!("{LOGGER_NAME}: Deleting schedule {schedule_id}");
                if !is_schedule_id_valid(schedule_id) {
                    return Err(Error::BadRequest(
                        "Schedule ID not valid".try_into().unwrap(),
                    ));
                }

                match panel.delete_schedule(schedule_id).await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        log::error!("{LOGGER_NAME}: {err}");
                        Err(err)
                    }
                }
            },
    )
}

/// Creates a router for the schedules collection endpoint
///
/// # Returns
/// * A router that handles requests for all schedules
pub fn schedules_router() -> impl MethodHandler<AppState> {
    get(|State(panel): State<&'static Panel>| async move {
        match panel.get_schedules().await {
            Ok(schedules) => Ok(Json(schedules)),
            Err(err) => {
                log::error!("{LOGGER_NAME}: {err}");
                Err(err)
            }
        }
    })
    .post(
        |State(panel): State<&'static Panel>, Json::<Vec<Schedule>>(schedules)| async move {
            for schedule in schedules {
                if let Err(err) = panel.set_schedule(schedule.id, schedule).await {
                    log::error!("{LOGGER_NAME}: {err}");
                    return Err(err);
                }
            }

            Ok(())
        },
    )
}

pub fn delete_all_router() -> impl MethodHandler<AppState> {
    post(|State(panel): State<&'static Panel>| async move {
        if let Err(err) = panel.delete_all().await {
            log::error!("{LOGGER_NAME}: {err}");
            return Err(err);
        }

        Ok(())
    })
}
pub fn ota_router() -> impl MethodHandler<AppState> {
    put_service(OverTheAirUpdate)
}

/// Checks if an ID is valid (A-Z)
///
/// # Arguments
/// * `id` - The ID to check
///
/// # Returns
/// * `true` if the ID is valid (A-Z)
/// * `false` otherwise
fn is_page_id_valid(id: char) -> bool {
    id >= 'A' && id <= 'Z'
}

/// Checks if an ID is valid page(A-Z)
///
/// # Arguments
/// * `id` - The ID to check
///
/// # Returns
/// * `true` if the ID is valid (A-Z)
/// * `false` otherwise
fn is_schedule_id_valid(id: char) -> bool {
    id >= 'A' && id <= 'E'
}
