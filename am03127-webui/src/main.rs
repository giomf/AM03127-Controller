use am03127_client::PanelClient;
use am03127_commands::{
    page::{Lagging, Leading, Page, WaitingModeAndSpeed, WaitingTime},
    realtime_clock::DateTime,
    schedule::{Schedule, ScheduleDateTime},
};
use anyhow::{Result, anyhow};
use leptos::{html, prelude::*, task::spawn_local};
use serde::{Serialize, de::DeserializeOwned};
use wasm_bindgen_futures::JsFuture;

const PAGE_IDS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const SCHEDULE_IDS: &str = "ABCDE";
const TABS: [&str; 2] = ["Content", "Device"];

/// Shared UI state: the notice bar, a counter that lists re-fetch on and, behind
/// am03127-server, the selected panel.
#[derive(Clone, Copy)]
struct Ctx {
    notice: RwSignal<Option<Result<String, String>>>,
    refresh: RwSignal<u32>,
    panel: RwSignal<Option<String>>,
}

/// A panel as listed by am03127-server's `/api/panels`.
#[cfg(feature = "server")]
#[derive(Clone, serde::Deserialize)]
struct PanelInfo {
    name: String,
    hidden: bool,
    floor: Option<String>,
    model: Option<am03127_client::config::Model>,
}

/// `None` is the panel serving this page; `Some(name)` a panel behind am03127-server.
fn client_for(panel: Option<&str>) -> PanelClient {
    let location = window().location();
    match panel {
        None => PanelClient::new(&location.host().unwrap()),
        Some(name) => PanelClient::with_base_url(format!(
            "{}/api/panel/{}",
            location.origin().unwrap(),
            js_sys::encode_uri_component(name)
        )),
    }
}

/// Client for the selected panel.
fn client() -> PanelClient {
    client_for(expect_context::<Ctx>().panel.get_untracked().as_deref())
}

/// The panels configured in am03127-server.
#[cfg(feature = "server")]
async fn fetch_panels() -> Result<Vec<PanelInfo>, String> {
    let response = gloo_net::http::Request::get("/api/panels")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(format!("HTTP {}", response.status()));
    }
    response.json().await.map_err(|e| e.to_string())
}

/// Runs an API call in the background, then shows `done` (or the error) and refreshes the lists.
fn run(done: &'static str, call: impl Future<Output = Result<()>> + 'static) {
    let ctx = expect_context::<Ctx>();
    ctx.notice.set(Some(Ok("Working…".into())));
    spawn_local(async move {
        match call.await {
            Ok(()) => {
                let notice = Some(Ok(done.to_string()));
                ctx.notice.set(notice.clone());
                ctx.refresh.update(|n| *n += 1);
                // Hide the toast again unless a newer one replaced it meanwhile.
                set_timeout(
                    move || {
                        if ctx.notice.get_untracked() == notice {
                            ctx.notice.set(None);
                        }
                    },
                    std::time::Duration::from_secs(3),
                );
            }
            Err(err) => ctx.notice.set(Some(Err(err.to_string()))),
        }
    });
}

/// Serde name of an enum variant, e.g. `ScrollLeft` -> "scroll_left".
fn name<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn parse<T: DeserializeOwned + Default>(name: &str) -> T {
    serde_json::from_value(serde_json::Value::String(name.into())).unwrap_or_default()
}

/// "scroll_left" -> "Scroll left"
fn pretty(name: &str) -> String {
    let text = name.replace('_', " ");
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// `<input type="datetime-local">` value -> panel time. The panel only stores two-digit years.
fn parse_local(value: &str) -> Option<ScheduleDateTime> {
    let (date, time) = value.split_once('T')?;
    let mut date = date.split('-').map(|n| n.parse::<u16>().ok());
    let mut time = time.split(':').map(|n| n.parse::<u8>().ok());
    Some(ScheduleDateTime {
        year: (date.next()?? % 100) as u8,
        month: date.next()?? as u8,
        day: date.next()?? as u8,
        hour: time.next()??,
        minute: time.next()??,
    })
}

fn format_local(dt: &ScheduleDateTime) -> String {
    format!(
        "20{:02}-{:02}-{:02}T{:02}:{:02}",
        dt.year, dt.month, dt.day, dt.hour, dt.minute
    )
}

fn browser_now() -> Result<DateTime> {
    let now = js_sys::Date::new_0();
    let month = now.get_month() as u8 + 1;
    let date = time::Date::from_calendar_date(
        now.get_full_year() as i32,
        time::Month::try_from(month)?,
        now.get_date() as u8,
    )?;
    Ok(DateTime {
        year: (now.get_full_year() % 100) as u8,
        week: date.iso_week(),
        month,
        day: now.get_date() as u8,
        hour: now.get_hours() as u8,
        minute: now.get_minutes() as u8,
        second: now.get_seconds() as u8,
    })
}

#[component]
fn Select(options: Vec<(String, String)>, value: RwSignal<String>) -> impl IntoView {
    view! {
        <select on:change=move |ev| value.set(event_target_value(&ev))>
            {options.into_iter().map(|(option, label)| {
                let current = option.clone();
                view! { <option value=option selected=move || value.get() == current>{label}</option> }
            }).collect_view()}
        </select>
    }
}

/// Dropdown entries for serde names, labelled for humans.
fn choices(names: impl IntoIterator<Item = String>) -> Vec<(String, String)> {
    names.into_iter().map(|n| (n.clone(), pretty(&n))).collect()
}

fn names<T: Serialize>(all: &[T]) -> Vec<String> {
    all.iter().map(name).collect()
}

/// IDs from `all` that are not taken yet.
fn free_ids(all: &str, used: impl Fn(char) -> bool) -> Vec<String> {
    all.chars().filter(|&c| !used(c)).map(String::from).collect()
}

/// The panel's waiting mode is speed x style (e.g. `middle_fast_song1`); the UI picks each separately.
const SPEEDS: [&str; 4] = ["fastest", "middle_fast", "middle_slow", "slowest"];
const STYLES: [&str; 5] = ["normal", "blinking", "song1", "song2", "song3"];

fn split_mode(mode: &WaitingModeAndSpeed) -> (String, String) {
    let name = name(mode);
    let (speed, style) = name.rsplit_once('_').unwrap_or(("fastest", "normal"));
    (speed.into(), style.into())
}

/// Waiting time steps are half seconds, starting at 0.5 s.
fn seconds(wait: u8) -> String {
    format!("{:.1} s", (wait as f32 + 1.0) / 2.0)
}

/// One table row. Existing pages show as text until "Edit"; the trailing new row
/// (`page: None`) is always in edit mode and offers only the free IDs.
#[component]
fn PageRow(page: Option<Page>, #[prop(optional)] free: Vec<String>) -> impl IntoView {
    let is_new = page.is_none();
    let first_free = free.first().and_then(|f| f.chars().next()).unwrap_or('A');
    let page = page.unwrap_or_else(|| {
        Page::new(
            first_free,
            String::new(),
            Leading::default(),
            Lagging::default(),
            WaitingModeAndSpeed::default(),
            WaitingTime::new(0),
        )
    });
    let page_id = page.id;

    let editing = RwSignal::new(is_new);
    let id = RwSignal::new(page.id.to_string());
    let message = RwSignal::new(String::new());
    let leading = RwSignal::new(String::new());
    let lagging = RwSignal::new(String::new());
    let speed = RwSignal::new(String::new());
    let style = RwSignal::new(String::new());
    let wait = RwSignal::new(0u8);

    let load = move |page: &Page| {
        let (page_speed, page_style) = split_mode(&page.waiting_mode_and_speed);
        message.set(page.message.clone());
        leading.set(name(&page.leading));
        lagging.set(name(&page.lagging));
        speed.set(page_speed);
        style.set(page_style);
        wait.set(page.waiting_time.value());
    };
    load(&page);
    let original = StoredValue::new(page);

    let save = move |_| {
        let page = Page::new(
            id.get().chars().next().unwrap_or('A'),
            message.get(),
            parse(&leading.get()),
            parse(&lagging.get()),
            parse(&format!("{}_{}", speed.get(), style.get())),
            WaitingTime::new(wait.get()),
        );
        let done = if is_new { "Page added" } else { "Page saved" };
        let client = client();
        run(done, async move { client.set_page(&page).await });
    };
    let cancel = move |_| {
        original.with_value(load);
        editing.set(false);
    };
    let delete = move |_| {
        let client = client();
        run("Page deleted", async move { client.delete_page(page_id).await })
    };

    move || {
        if !editing.get() {
            return view! {
                <tr>
                    <td>{page_id.to_string()}</td>
                    <td>{message.get()}</td>
                    <td>{pretty(&leading.get())}</td>
                    <td>{pretty(&lagging.get())}</td>
                    <td>{pretty(&speed.get())}</td>
                    <td>{pretty(&style.get())}</td>
                    <td>{seconds(wait.get())}</td>
                    <td class="actions">
                        <button on:click=move |_| editing.set(true)>"Edit"</button>
                        <button class="danger" on:click=delete>"Delete"</button>
                    </td>
                </tr>
            }
            .into_any();
        }
        let id_cell = if is_new {
            view! { <Select options=choices(free.clone()) value=id /> }.into_any()
        } else {
            page_id.to_string().into_any()
        };
        view! {
            <tr class="editing">
                <td>{id_cell}</td>
                <td><input type="text" placeholder="Message" bind:value=message /></td>
                <td><Select options=choices(names(Leading::ALL)) value=leading /></td>
                <td><Select options=choices(names(Lagging::ALL)) value=lagging /></td>
                <td><Select options=choices(SPEEDS.map(String::from)) value=speed /></td>
                <td><Select options=choices(STYLES.map(String::from)) value=style /></td>
                <td>
                    <select on:change=move |ev| wait.set(event_target_value(&ev).parse().unwrap_or(0))>
                        {(0..=25u8).map(|w| view! {
                            <option value=w.to_string() selected=move || wait.get() == w>{seconds(w)}</option>
                        }).collect_view()}
                    </select>
                </td>
                <td class="actions">
                    {if is_new {
                        view! { <button class="primary" on:click=save>"Add"</button> }.into_any()
                    } else {
                        view! {
                            <button class="primary" on:click=save>"Save"</button>
                            <button on:click=cancel>"Cancel"</button>
                        }
                        .into_any()
                    }}
                </td>
            </tr>
        }
        .into_any()
    }
}

#[component]
fn Pages() -> impl IntoView {
    let ctx = expect_context::<Ctx>();
    let pages = LocalResource::new(move || {
        ctx.refresh.track();
        let client = client();
        async move { client.get_pages().await.map_err(|e| e.to_string()) }
    });

    view! {
        <section>
            <h2>"Pages"</h2>
            <Suspense fallback=|| view! { <p>"Loading…"</p> }>
                {move || pages.get().map(|result| match result {
                    Ok(mut list) => {
                        list.sort_by_key(|p| p.id);
                        let free = free_ids(PAGE_IDS, |c| list.iter().any(|p| p.id == c));
                        view! {
                            <div class="table">
                                <table>
                                    <thead><tr>
                                        <th>"ID"</th><th>"Message"</th><th>"In"</th><th>"Out"</th>
                                        <th>"Speed"</th><th>"Style"</th><th>"Wait"</th><th></th>
                                    </tr></thead>
                                    <tbody>
                                        {list.into_iter().map(|page| view! { <PageRow page=Some(page) /> }).collect_view()}
                                        {(!free.is_empty()).then(|| view! { <PageRow page=None free=free /> })}
                                    </tbody>
                                </table>
                            </div>
                        }
                        .into_any()
                    }
                    Err(err) => view! { <p class="error">{err}</p> }.into_any(),
                })}
            </Suspense>
            <FormattingHelp />
        </section>
    }
}

/// Same pattern as [`PageRow`].
#[component]
fn ScheduleRow(schedule: Option<Schedule>, #[prop(optional)] free: Vec<String>) -> impl IntoView {
    let is_new = schedule.is_none();
    let first_free = free.first().and_then(|f| f.chars().next()).unwrap_or('A');
    let schedule = schedule.unwrap_or_else(|| {
        Schedule::new(first_free, ScheduleDateTime::default(), ScheduleDateTime::default(), String::new())
    });
    let schedule_id = schedule.id;

    let editing = RwSignal::new(is_new);
    let id = RwSignal::new(schedule.id.to_string());
    let from = RwSignal::new(String::new());
    let to = RwSignal::new(String::new());
    let pages = RwSignal::new(String::new());

    let load = move |schedule: &Schedule| {
        // A blank new row should show empty pickers, not 2000-00-00.
        let local = |dt: &ScheduleDateTime| if is_new { String::new() } else { format_local(dt) };
        from.set(local(&schedule.from));
        to.set(local(&schedule.to));
        pages.set(schedule.pages.clone());
    };
    load(&schedule);
    let original = StoredValue::new(schedule);

    let save = move |_| {
        let ctx = expect_context::<Ctx>();
        let (Some(start), Some(end)) = (parse_local(&from.get()), parse_local(&to.get())) else {
            ctx.notice.set(Some(Err("Pick a start and an end time".into())));
            return;
        };
        let page_ids = pages.get().to_uppercase();
        if page_ids.is_empty() || !page_ids.chars().all(|c| PAGE_IDS.contains(c)) {
            ctx.notice.set(Some(Err("Pages must be page IDs in display order, e.g. ABC".into())));
            return;
        }
        let schedule = Schedule::new(id.get().chars().next().unwrap_or('A'), start, end, page_ids);
        let done = if is_new { "Schedule added" } else { "Schedule saved" };
        let client = client();
        run(done, async move { client.set_schedule(&schedule).await });
    };
    let cancel = move |_| {
        original.with_value(load);
        editing.set(false);
    };
    let delete = move |_| {
        let client = client();
        run("Schedule deleted", async move { client.delete_schedule(schedule_id).await })
    };

    move || {
        if !editing.get() {
            return view! {
                <tr>
                    <td>{schedule_id.to_string()}</td>
                    <td>{from.get().replace('T', " ")}</td>
                    <td>{to.get().replace('T', " ")}</td>
                    <td>{pages.get()}</td>
                    <td class="actions">
                        <button on:click=move |_| editing.set(true)>"Edit"</button>
                        <button class="danger" on:click=delete>"Delete"</button>
                    </td>
                </tr>
            }
            .into_any();
        }
        let id_cell = if is_new {
            view! { <Select options=choices(free.clone()) value=id /> }.into_any()
        } else {
            schedule_id.to_string().into_any()
        };
        view! {
            <tr class="editing">
                <td>{id_cell}</td>
                <td><input type="datetime-local" bind:value=from /></td>
                <td><input type="datetime-local" bind:value=to /></td>
                <td><input type="text" placeholder="ABC" bind:value=pages /></td>
                <td class="actions">
                    {if is_new {
                        view! { <button class="primary" on:click=save>"Add"</button> }.into_any()
                    } else {
                        view! {
                            <button class="primary" on:click=save>"Save"</button>
                            <button on:click=cancel>"Cancel"</button>
                        }
                        .into_any()
                    }}
                </td>
            </tr>
        }
        .into_any()
    }
}

#[component]
fn Schedules() -> impl IntoView {
    let ctx = expect_context::<Ctx>();
    let schedules = LocalResource::new(move || {
        ctx.refresh.track();
        let client = client();
        async move { client.get_schedules().await.map_err(|e| e.to_string()) }
    });

    view! {
        <section>
            <h2>"Schedules"</h2>
            <Suspense fallback=|| view! { <p>"Loading…"</p> }>
                {move || schedules.get().map(|result| match result {
                    Ok(mut list) => {
                        list.sort_by_key(|s| s.id);
                        let free = free_ids(SCHEDULE_IDS, |c| list.iter().any(|s| s.id == c));
                        view! {
                            <div class="table">
                                <table>
                                    <thead><tr>
                                        <th>"ID"</th><th>"From"</th><th>"To"</th><th>"Pages"</th><th></th>
                                    </tr></thead>
                                    <tbody>
                                        {list.into_iter().map(|s| view! { <ScheduleRow schedule=Some(s) /> }).collect_view()}
                                        {(!free.is_empty()).then(|| view! { <ScheduleRow schedule=None free=free /> })}
                                    </tbody>
                                </table>
                            </div>
                        }
                        .into_any()
                    }
                    Err(err) => view! { <p class="error">{err}</p> }.into_any(),
                })}
            </Suspense>
        </section>
    }
}

#[component]
fn FormattingHelp() -> impl IntoView {
    use am03127_commands::formatting::{Clock, ColumnStart, Font};
    let tags = [
        (Font::Normal.to_string(), "Normal font (5x7)"),
        (Font::Bold.to_string(), "Bold font (6x7)"),
        (Font::Narrow.to_string(), "Narrow font (4x7)"),
        (Font::Large.to_string(), "Large font (7x13), 16 px high panels only"),
        (Font::Long.to_string(), "Long font (5x8), panels higher than 7 px only"),
        (Clock::Date.to_string(), "Current date (DD/MM/YY)"),
        (Clock::Time.to_string(), "Current time (hh:mm)"),
        (ColumnStart(0x10).to_string(), "Start text at column NN (hex), here column 16"),
    ];

    view! {
        <details>
            <summary>"Special characters"</summary>
            <div class="table">
                <table>
                    <thead><tr><th>"Code"</th><th>"Effect"</th></tr></thead>
                    <tbody>
                        {tags.into_iter().map(|(code, effect)| view! {
                            <tr><td><code>{code}</code></td><td>{effect}</td></tr>
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </details>
    }
}

#[component]
fn Device() -> impl IntoView {
    let ctx = expect_context::<Ctx>();
    let status = LocalResource::new(move || {
        ctx.refresh.track();
        let client = client();
        async move { client.get_status().await.map_err(|e| e.to_string()) }
    });
    let file = NodeRef::<html::Input>::new();
    let file_name = RwSignal::new(None::<String>);

    let sync_clock = move |_| {
        let client = client();
        run("Clock set to this browser's time", async move {
            client.set_clock(&browser_now()?).await
        })
    };

    let upload = move |_| {
        let Some(selected) = file.get().and_then(|input| input.files()).and_then(|f| f.get(0)) else {
            expect_context::<Ctx>().notice.set(Some(Err("Choose a firmware file first".into())));
            return;
        };
        let client = client();
        run("Firmware uploaded, the panel is restarting", async move {
            let buffer = JsFuture::from(selected.array_buffer())
                .await
                .map_err(|e| anyhow!("could not read file: {e:?}"))?;
            client.update_firmware(&js_sys::Uint8Array::new(&buffer).to_vec()).await
        });
    };

    let reset = move |_| {
        if window().confirm_with_message("Delete all pages and schedules?").unwrap_or(false) {
            let client = client();
            run("All pages and schedules deleted", async move { client.reset().await });
        }
    };

    view! {
        <section class="cards">
            <div class="card">
                <h2>"Status"</h2>
                <Suspense fallback=|| view! { <p>"Loading…"</p> }>
                    {move || status.get().map(|result| match result {
                        Ok(s) => view! {
                            <dl class="status">
                                <dt>"Firmware"</dt><dd>{s.version.clone()}</dd>
                                <dt>"Built"</dt><dd>{format!("{} {} UTC", s.build_date, s.build_time)}</dd>
                                <dt>"Uptime"</dt><dd>{s.uptime()}</dd>
                                <dt>"Wi-Fi"</dt>
                                <dd>{s.rssi.map(|rssi| format!("{rssi} dBm")).unwrap_or_else(|| "not measured yet".into())}</dd>
                                <dt>"Memory"</dt>
                                <dd>{format!("{} KB free, {} KB used", s.heap_free / 1024, s.heap_used / 1024)}</dd>
                                <dt>"Panel"</dt>
                                <dd class:error=s.panel_ok == Some(false)>
                                    {match s.panel_ok {
                                        Some(true) => "answers",
                                        Some(false) => "not responding",
                                        None => "no command sent yet",
                                    }}
                                </dd>
                            </dl>
                        }.into_any(),
                        Err(err) => view! { <p class="error">"Unreachable: "{err}</p> }.into_any(),
                    })}
                </Suspense>
                <button on:click=move |_| status.refetch()>"Refresh"</button>
            </div>
            <div class="card">
                <h2>"Clock"</h2>
                <p>"Set the panel's clock to the current time of this browser."</p>
                <button class="primary" on:click=sync_clock>"Sync clock"</button>
            </div>
            <div class="card">
                <h2>"Firmware update"</h2>
                <p>"Upload a firmware image (.bin). The panel restarts after the upload."</p>
                <label class="file">
                    <input
                        type="file" accept=".bin" node_ref=file
                        on:change=move |_| file_name.set(
                            file.get().and_then(|i| i.files()).and_then(|f| f.get(0))
                                .map(|f| format!("{} ({} KB)", f.name(), (f.size() / 1024.0).round()))
                        )
                    />
                    {move || file_name.get().unwrap_or_else(|| "Choose file…".into())}
                </label>
                <button class="primary" disabled=move || file_name.get().is_none() on:click=upload>"Upload"</button>
            </div>
            <div class="card danger-zone">
                <h2>"Danger zone"</h2>
                <p>"Delete all pages and schedules from the panel. This cannot be undone."</p>
                <button class="danger" on:click=reset>"Delete everything"</button>
            </div>
        </section>
    }
}

/// Panel picker behind am03127-server: click a panel to edit it. Panels are grouped by
/// floor, in the order the floors first appear in panel-config.toml.
#[cfg(feature = "server")]
#[component]
fn PanelBar(panels: Vec<PanelInfo>) -> impl IntoView {
    let mut floors: Vec<(Option<String>, Vec<PanelInfo>)> = Vec::new();
    for panel in panels {
        match floors.iter_mut().find(|(floor, _)| *floor == panel.floor) {
            Some((_, group)) => group.push(panel),
            None => floors.push((panel.floor.clone(), vec![panel])),
        }
    }

    floors
        .into_iter()
        .map(|(floor, panels)| {
            view! {
                {floor.map(|floor| view! { <h3 class="floor">{floor}</h3> })}
                <div class="panels">
                    {panels.into_iter().map(|panel| view! { <PanelChip panel /> }).collect_view()}
                </div>
            }
        })
        .collect_view()
}

/// One panel in the [`PanelBar`] with its online status and firmware version.
#[cfg(feature = "server")]
#[component]
fn PanelChip(panel: PanelInfo) -> impl IntoView {
    let ctx = expect_context::<Ctx>();
    let name = panel.name.clone();
    let status = LocalResource::new({
        let name = name.clone();
        move || {
            let client = client_for(Some(&name));
            async move { client.get_status().await.map_err(|e| e.to_string()) }
        }
    });
    let (is_current, click) = (name.clone(), name);

    view! {
        <div
            class="panel"
            class:active=move || ctx.panel.get().as_deref() == Some(is_current.as_str())
            on:click=move |_| {
                ctx.panel.set(Some(click.clone()));
                ctx.refresh.update(|n| *n += 1);
            }
        >
            <strong>
                {panel.name}{panel.hidden.then_some(" (hidden)")}
                {panel.model.map(|model| {
                    use am03127_client::config::Model;
                    let (class, title) = match model {
                        Model::Long => ("model long", "Long panel"),
                        Model::Color => ("model color", "Color panel"),
                    };
                    view! { <span class=class title=title></span> }
                })}
            </strong>
            {move || match status.get() {
                None => view! { <small>"…"</small> }.into_any(),
                Some(Ok(info)) => view! {
                    <small title=info.details().join("\n")>
                        <span class="dot online">"●"</span>" "{info.version.clone()}
                        {(info.panel_ok == Some(false)).then(|| view! {
                            <span class="error">" · panel not responding"</span>
                        })}
                    </small>
                }.into_any(),
                Some(Err(_)) => view! {
                    <small><span class="dot offline">"●"</span>" offline"</small>
                }.into_any(),
            }}
        </div>
    }
}

/// Tabs and their content for the selected panel.
#[component]
fn Body() -> impl IntoView {
    let tab = RwSignal::new(TABS[0]);
    view! {
        <nav>
            {TABS.iter().map(|&label| view! {
                <button class:active=move || tab.get() == label on:click=move |_| tab.set(label)>
                    {label}
                </button>
            }).collect_view()}
        </nav>
        <main>
            {move || match tab.get() {
                "Device" => view! { <Device /> }.into_any(),
                _ => view! { <Pages /> <Schedules /> }.into_any(),
            }}
        </main>
    }
}

/// Built with the `server` feature for am03127-server: loads the panel list and selects the
/// first visible panel before anything talks to a panel.
#[cfg(feature = "server")]
fn body() -> impl IntoView {
    let ctx = expect_context::<Ctx>();
    let panels = LocalResource::new(move || async move {
        let panels = fetch_panels().await?;
        let first = panels.iter().find(|p| !p.hidden).or(panels.first());
        let first = first.ok_or("panel-config.toml has no panels")?;
        ctx.panel.set(Some(first.name.clone()));
        Ok::<_, String>(panels)
    });
    view! {
        <Suspense fallback=|| view! { <p>"Loading…"</p> }>
            {move || panels.get().map(|result| match result {
                Ok(list) => view! { <PanelBar panels=list /> <Body /> }.into_any(),
                Err(err) => view! { <p class="error">"Could not load the panels: "{err}</p> }.into_any(),
            })}
        </Suspense>
    }
}

/// Built without the `server` feature, the UI is served by the panel it controls.
#[cfg(not(feature = "server"))]
fn body() -> impl IntoView {
    let status = LocalResource::new(|| {
        let client = client();
        async move { client.get_status().await.map_err(|e| e.to_string()) }
    });
    view! {
        <p class="status">
            <Suspense fallback=|| "">
                {move || status.get().map(|s| match s {
                    Ok(info) => format!("Firmware {} ({} {})", info.version, info.build_date, info.build_time),
                    Err(err) => format!("Panel unreachable: {err}"),
                })}
            </Suspense>
        </p>
        <Body />
    }
}

#[component]
fn App() -> impl IntoView {
    let ctx = Ctx {
        notice: RwSignal::new(None),
        refresh: RwSignal::new(0),
        panel: RwSignal::new(None),
    };
    provide_context(ctx);

    view! {
        <header>
            <h1>"AM03127-Controller"</h1>
        </header>
        {move || ctx.notice.get().map(|notice| {
            let (class, text) = match notice {
                Ok(text) => ("toast", text),
                Err(text) => ("toast error", format!("{text} (click to dismiss)")),
            };
            view! { <div class=class role="status" on:click=move |_| ctx.notice.set(None)>{text}</div> }
        })}
        {body()}
    }
}

fn main() {
    leptos::mount::mount_to_body(App);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datetime_local_roundtrip() {
        let dt = parse_local("2026-11-05T14:30").unwrap();
        assert_eq!((dt.year, dt.month, dt.day, dt.hour, dt.minute), (26, 11, 5, 14, 30));
        assert_eq!(format_local(&dt), "2026-11-05T14:30");
        assert!(parse_local("").is_none());
        assert!(parse_local("2026-11-05").is_none());
    }

    #[test]
    fn waiting_mode_split() {
        assert_eq!(
            split_mode(&WaitingModeAndSpeed::MiddleFastSong1),
            ("middle_fast".into(), "song1".into())
        );
        for mode in WaitingModeAndSpeed::ALL {
            let (speed, style) = split_mode(mode);
            assert_eq!(name(&parse::<WaitingModeAndSpeed>(&format!("{speed}_{style}"))), name(mode));
        }
    }

    #[test]
    fn enum_names_roundtrip() {
        assert_eq!(name(&Leading::ScrollLeft), "scroll_left");
        assert!(matches!(parse::<Leading>("scroll_left"), Leading::ScrollLeft));
        assert_eq!(pretty("scroll_left"), "Scroll left");
    }
}
