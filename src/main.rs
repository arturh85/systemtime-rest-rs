#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;

use std::process::Command;
use chrono::{Datelike, DateTime, Duration, Timelike, Utc};
use rocket::http::Status;
use rocket::response::status;
use rocket::Request;
use rocket::response::content::Html;

fn set_system_time(time: DateTime<Utc>) {
    let system_time = windows_sys::Win32::Foundation::SYSTEMTIME {
        wYear: time.year() as u16,
        wMonth: time.month() as u16,
        wDayOfWeek: time.weekday().num_days_from_monday() as u16,
        wDay: time.day() as u16,
        wHour: time.hour() as u16,
        wMinute: time.minute() as u16,
        wSecond: time.second() as u16,
        wMilliseconds: 0
    };
    unsafe {
        windows_sys::Win32::System::SystemInformation::SetSystemTime(&system_time);
    }
}

/// Adds given amount of hours to system clock
#[openapi(tag = "Time")]
#[get("/add/<hours>")]
fn add_hours(hours: i64) -> &'static str {
    if hours.abs() > 365 * 24 * 10 {
        "dont want to skip more than a decade at a time"
    } else {
        Command::new("net")
            .args(["stop", "w32time"])
            .output()
            .expect("failed to execute process");
        match Utc::now().checked_add_signed(Duration::hours(hours as i64)) {
            Some(time) => {
                set_system_time(time);
                "ok"
            }
            None => "invalid"
        }
    }
}

/// Resets time back to original
#[openapi(tag = "Time")]
#[get("/reset")]
fn reset() -> &'static str {
    Command::new("net")
        .args(["start", "w32time"])
        .output()
        .expect("failed to execute process");
    Command::new("w32tm")
        .args(["/resync"])
        .output()
        .expect("failed to execute process");
    "ok"
}

#[get("/")]
fn index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

#[catch(default)]
fn default_catcher(status: Status, req: &Request<'_>) -> status::Custom<String> {
    let msg = format!("{} ({})", status, req.uri());
    status::Custom(status, msg)
}

#[catch(404)]
fn general_not_found() -> String {
    "Not found".into()
}

#[async_std::main]
async fn main() {
    if !is_elevated::is_elevated() {
        eprintln!(
            "ERROR: Must be started as Administrator"
        );
        return;
    }
    let port = 4334;
    let figment = rocket::Config::figment()
        .merge(("port", port));
    rocket::custom(figment)
        .mount(
            "/",
            rocket::routes![index],
        )
        .mount(
            "/",
            routes_with_openapi![reset, add_hours],
        )
        .register("/", rocket::catchers![general_not_found, default_catcher])
        .launch()
        .await
        .unwrap();
}