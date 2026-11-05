extern crate alloc;
use alloc::vec::Vec;

use am03127_commands::{page::Page, realtime_clock::DateTime, schedule::Schedule, status::Status};
#[cfg(feature = "web_interface")]
use picoserve::routing::get_service;
use picoserve::{
    extract::{Json, State},
    routing::{MethodHandler, get, post, put_service},
};

use super::AppState;
use crate::{
    error::Error,
    panel::{Panel, is_page_id_valid, is_schedule_id_valid},
    server::ota::OverTheAirUpdate,
};

/// Logger name for router-related log messages
const LOGGER_NAME: &str = "Router";

/// Creates a router for the /status endpoint
///
/// # Returns
/// * A router that returns build info and health (uptime, Wi-Fi, heap, panel) as JSON
pub fn status_router() -> impl MethodHandler<AppState> {
    get(|| async move {
        let rssi = crate::WIFI_RSSI.load(core::sync::atomic::Ordering::Relaxed);
        Json(Status {
            version: env!("BUILD_VERSION").into(),
            build_time: env!("BUILD_TIME").into(),
            build_date: env!("BUILD_DATE").into(),
            uptime_secs: embassy_time::Instant::now().as_secs(),
            rssi: (rssi != i32::MIN).then_some(rssi),
            heap_free: esp_alloc::HEAP.free() as u32,
            heap_used: esp_alloc::HEAP.used() as u32,
            panel_ok: crate::uart::panel_ok(),
        })
    })
}

/// Creates a router for one gzipped file of the web UI
///
/// # Returns
/// * A router that serves the file with `Content-Encoding: gzip`
#[cfg(feature = "web_interface")]
pub fn webui_router(
    content_type: &'static str,
    body: &'static [u8],
) -> impl MethodHandler<AppState> {
    const GZIP: &[(&str, &str)] = &[("Content-Encoding", "gzip")];
    get_service(picoserve::response::File::with_content_type_and_headers(
        content_type,
        body,
        GZIP,
    ))
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
            if page.id != page_id {
                return Err(Error::BadRequest(
                    "Page ID in path and body differ".try_into().unwrap(),
                ));
            }

            match panel.set_page(page).await {
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
            // Check all first so a bad entry does not leave a half-applied upload.
            if pages.iter().any(|page| !is_page_id_valid(page.id)) {
                return Err(Error::BadRequest("Page ID not valid".try_into().unwrap()));
            }
            for page in pages {
                if let Err(err) = panel.set_page(page).await {
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
                log::info!("{LOGGER_NAME}: Getting schedule \"{schedule_id}\"");
                if !is_schedule_id_valid(schedule_id) {
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

                if schedule.id != schedule_id {
                    return Err(Error::BadRequest(
                        "Schedule ID in path and body differ".try_into().unwrap(),
                    ));
                }

                match panel.set_schedule(schedule).await {
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
            // Check all first so a bad entry does not leave a half-applied upload.
            if schedules
                .iter()
                .any(|schedule| !is_schedule_id_valid(schedule.id))
            {
                return Err(Error::BadRequest(
                    "Schedule ID not valid".try_into().unwrap(),
                ));
            }
            for schedule in schedules {
                if let Err(err) = panel.set_schedule(schedule).await {
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
