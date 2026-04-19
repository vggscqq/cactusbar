use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

fn weather_icon(code: i32) -> &'static str {
    match code {
        0 => "☀", 1..=3 => "⛅", 45..=48 => "🌫", 51..=67 => "🌧",
        71..=77 => "❄", 80..=82 => "🌦", 85..=86 => "🌨", 95..=99 => "⛈", _ => "🌡",
    }
}

#[derive(Clone, Debug)]
struct WeatherData {
    current_text: String,
    forecast: Vec<String>,
}

fn read_weather(lat: &str, lon: &str) -> Option<WeatherData> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code&daily=weather_code,temperature_2m_max&timezone=auto&forecast_days=7",
        lat, lon
    );
    let json = fetch_json(&url)?;
    let temp = json["current"]["temperature_2m"].as_f64()?;
    let code = json["current"]["weather_code"].as_i64().unwrap_or(0) as i32;
    let current_text = format!("{} {:.0}°", weather_icon(code), temp);

    let days = json["daily"]["time"].as_array().cloned().unwrap_or_default();
    let temps = json["daily"]["temperature_2m_max"].as_array().cloned().unwrap_or_default();
    let codes = json["daily"]["weather_code"].as_array().cloned().unwrap_or_default();

    let forecast: Vec<String> = days.iter().zip(temps.iter()).zip(codes.iter())
        .map(|((d, t), c)| {
            let day = d.as_str().unwrap_or("");
            let temp = t.as_f64().unwrap_or(0.0);
            let code = c.as_i64().unwrap_or(0) as i32;
            format!("{} {} {:.0}°", day, weather_icon(code), temp)
        })
        .collect();

    Some(WeatherData { current_text, forecast })
}

pub fn new_weather(cfg: &crate::config::Config) -> gtk4::Box {
    let module = TextModule::new("custom-weather");

    let popover = gtk4::Popover::new();
    popover.add_css_class("status-popup");
    popover.set_has_arrow(false);
    popover.set_autohide(true);
    popover.set_parent(&module.container);

    let menu = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    menu.set_widget_name("weather-menu");
    popover.set_child(Some(&menu));

    let title_text = if cfg.weather.location.is_empty() {
        "Weather".to_string()
    } else {
        format!("Weather — {}", cfg.weather.location)
    };
    let title = gtk4::Label::new(Some(&title_text));
    title.set_widget_name("weather-menu-title");
    title.set_xalign(0.0);
    menu.append(&title);

    let mut forecast_rows: Vec<gtk4::Label> = Vec::with_capacity(7);
    for _ in 0..7 {
        let row = gtk4::Label::new(None);
        row.add_css_class("weather-forecast-row");
        row.set_xalign(0.0);
        menu.append(&row);
        forecast_rows.push(row);
    }

    let motion = gtk4::EventControllerMotion::new();
    let popover_weak = popover.downgrade();
    motion.connect_enter(move |_, _, _| {
        if let Some(p) = popover_weak.upgrade() { p.popup(); }
    });
    let popover_weak2 = popover.downgrade();
    motion.connect_leave(move |_| {
        if let Some(p) = popover_weak2.upgrade() {
            glib::timeout_add_local_once(Duration::from_millis(100), move || { p.popdown(); });
        }
    });
    module.container.add_controller(motion);

    let on_click = cfg.weather.on_click.clone();
    attach_click(&module.container, move || { run_detached(&on_click); }, || {});

    let lat = cfg.weather.lat.clone();
    let lon = cfg.weather.lon.clone();
    let (tx, rx) = async_channel::bounded::<Option<WeatherData>>(1);

    let do_fetch = {
        let lat = lat.clone();
        let lon = lon.clone();
        let tx = tx.clone();
        move || {
            let lat = lat.clone();
            let lon = lon.clone();
            let tx = tx.clone();
            std::thread::spawn(move || {
                let result = read_weather(&lat, &lon);
                tx.send_blocking(result).ok();
            });
        }
    };

    // Main thread receiver
    let container_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();
    let row_weaks: Vec<_> = forecast_rows.iter().map(|r| r.downgrade()).collect();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(result) = rx.recv().await {
            let data = match result { Some(d) => d, None => continue };
            if let Some(l) = label_weak.upgrade() { l.set_label(&data.current_text); }
            if let Some(c) = container_weak.upgrade() { c.set_visible(!data.current_text.is_empty()); }
            for (i, rw) in row_weaks.iter().enumerate() {
                if let Some(row) = rw.upgrade() {
                    if i < data.forecast.len() {
                        row.set_label(&data.forecast[i]);
                        row.set_visible(true);
                    } else {
                        row.set_visible(false);
                    }
                }
            }
        }
    });

    do_fetch();

    let container_weak2 = module.container.downgrade();
    glib::timeout_add_local(Duration::from_secs(30 * 60), move || {
        if container_weak2.upgrade().is_none() { return glib::ControlFlow::Break; }
        do_fetch();
        glib::ControlFlow::Continue
    });

    module.container
}
