use gtk4::prelude::*;
use glib::prelude::*;
use chrono::Datelike;
use std::time::Duration;
use super::helpers::*;

pub fn new_date_clock() -> gtk4::Box {
    let module = TextModule::new("clock-date");

    let (popup_date, menu) = make_popup();
    menu.set_widget_name("calendar-menu");
    menu.set_spacing(6);

    let month_nav = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    month_nav.set_halign(gtk4::Align::Fill);
    month_nav.set_hexpand(true);
    month_nav.set_widget_name("calendar-menu-month-nav");

    let prev_btn = gtk4::Button::with_label("<");
    prev_btn.set_halign(gtk4::Align::Start);
    month_nav.append(&prev_btn);

    let month_label = gtk4::Label::new(None);
    month_label.set_widget_name("calendar-menu-month");
    month_label.set_xalign(0.5);
    month_label.set_hexpand(true);
    month_nav.append(&month_label);

    let next_btn = gtk4::Button::with_label(">");
    next_btn.set_halign(gtk4::Align::End);
    month_nav.append(&next_btn);
    menu.append(&month_nav);

    let weekday_row = gtk4::Grid::new();
    weekday_row.set_column_homogeneous(true);
    weekday_row.set_column_spacing(4);
    weekday_row.set_widget_name("calendar-menu-weekdays");
    menu.append(&weekday_row);

    for (i, wd) in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].iter().enumerate() {
        let lbl = gtk4::Label::new(Some(wd));
        lbl.add_css_class("calendar-weekday");
        weekday_row.attach(&lbl, i as i32, 0, 1, 1);
    }

    let calendar_grid = gtk4::Grid::new();
    calendar_grid.set_column_homogeneous(true);
    calendar_grid.set_row_spacing(4);
    calendar_grid.set_column_spacing(4);
    calendar_grid.set_widget_name("calendar-menu-grid");
    menu.append(&calendar_grid);

    let mut day_cells: Vec<gtk4::Label> = Vec::with_capacity(42);
    for row in 0..6i32 {
        for col in 0..7i32 {
            let cell = gtk4::Label::new(None);
            cell.add_css_class("calendar-day");
            calendar_grid.attach(&cell, col, row, 1, 1);
            day_cells.push(cell);
        }
    }

    let shown_year = std::rc::Rc::new(std::cell::Cell::new(0i32));
    let shown_month = std::rc::Rc::new(std::cell::Cell::new(0i32));
    let today_rc: std::rc::Rc<std::cell::Cell<(i32, i32, i32)>> = std::rc::Rc::new(std::cell::Cell::new((0, 0, 0)));

    let now = chrono::Local::now();
    shown_year.set(now.year());
    shown_month.set(now.month() as i32);
    today_rc.set((now.year(), now.month() as i32, now.day() as i32));

    let update_calendar = {
        let shown_year = shown_year.clone();
        let shown_month = shown_month.clone();
        let today_rc = today_rc.clone();
        let month_label = month_label.clone();
        let day_cells = day_cells.clone();
        let module_label = module.label.clone();
        let module_container = module.container.clone();

        std::rc::Rc::new(move || {
            let (ty, tm, td) = today_rc.get();
            let sy = shown_year.get();
            let sm = shown_month.get();

            let now_local = chrono::Local::now();
            let display = now_local.format("%-d %b %Y").to_string();
            module_label.set_label(&display);
            module_container.set_visible(true);

            let first_day = chrono::NaiveDate::from_ymd_opt(sy, sm as u32, 1).unwrap_or_default();
            let month_name = first_day.format("%B %Y").to_string();
            month_label.set_label(&month_name);

            let next_month = if sm == 12 {
                chrono::NaiveDate::from_ymd_opt(sy + 1, 1, 1).unwrap_or_default()
            } else {
                chrono::NaiveDate::from_ymd_opt(sy, sm as u32 + 1, 1).unwrap_or_default()
            };
            let days_in_month = (next_month - first_day).num_days() as i32;

            use chrono::Datelike;
            let first_weekday = first_day.weekday().num_days_from_monday() as i32;
            let current_day = if sy == ty && sm == tm { td } else { -1 };

            for cell in &day_cells {
                cell.set_label("");
                cell.remove_css_class("today");
            }
            for day in 1..=days_in_month {
                let idx = (first_weekday + day - 1) as usize;
                if idx < day_cells.len() {
                    day_cells[idx].set_label(&format!("{:2}", day));
                    if day == current_day { day_cells[idx].add_css_class("today"); }
                }
            }
        })
    };

    (update_calendar)();

    attach_hover_popup(&module.container, &popup_date, || {});

    {
        let sy = shown_year.clone(); let sm = shown_month.clone(); let uc = update_calendar.clone();
        prev_btn.connect_clicked(move |_| {
            let (y, m) = (sy.get(), sm.get());
            if m == 1 { sy.set(y - 1); sm.set(12); } else { sm.set(m - 1); }
            (uc)();
        });
    }
    {
        let sy = shown_year.clone(); let sm = shown_month.clone(); let uc = update_calendar.clone();
        next_btn.connect_clicked(move |_| {
            let (y, m) = (sy.get(), sm.get());
            if m == 12 { sy.set(y + 1); sm.set(1); } else { sm.set(m + 1); }
            (uc)();
        });
    }

    let module_weak = module.container.downgrade();
    let today_rc2 = today_rc.clone();
    let sy2 = shown_year.clone();
    let sm2 = shown_month.clone();
    let uc2 = update_calendar.clone();
    glib::timeout_add_local(Duration::from_secs(1), move || {
        if module_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        let now = chrono::Local::now();
        today_rc2.set((now.year(), now.month() as i32, now.day() as i32));
        let sy = sy2.get(); let sm = sm2.get();
        let (ty, tm, _) = today_rc2.get();
        if sy == ty && sm == tm { (uc2)(); }
        glib::ControlFlow::Continue
    });

    module.container
}

pub fn new_time_clock(cfg: &crate::config::Config) -> gtk4::Box {
    let module = TextModule::new("clock-time");

    let (popup_time, menu) = make_popup();
    menu.set_widget_name("world-clock-menu");

    let title = gtk4::Label::new(Some("World Clocks"));
    title.set_widget_name("world-clock-menu-title");
    title.set_xalign(0.0);
    menu.append(&title);

    let clocks = cfg.world_clocks.clone();
    let mut rows: Vec<gtk4::Label> = Vec::with_capacity(clocks.len());
    for _ in &clocks {
        let row = gtk4::Label::new(None);
        row.add_css_class("world-clock-row");
        row.set_xalign(0.0);
        menu.append(&row);
        rows.push(row);
    }

    let on_click_cmd = if cfg.clocks.on_click.is_empty() {
        "gnome-clocks".to_string()
    } else {
        cfg.clocks.on_click.clone()
    };

    attach_hover_popup(&module.container, &popup_time, || {});

    attach_click(&module.container, || {}, move || { run_detached(&on_click_cmd); });

    let module_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();

    // Show current time immediately so the clock is visible on startup.
    {
        let now = chrono::Local::now();
        module.label.set_label(&now.format("%H:%M").to_string());
        module.container.set_visible(true);
    }

    glib::timeout_add_local(Duration::from_secs(1), move || {
        if module_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        let label = match label_weak.upgrade() { Some(l) => l, None => return glib::ControlFlow::Break };

        let now = chrono::Local::now();
        let time_str = now.format("%H:%M").to_string();
        label.set_label(&time_str);
        if let Some(container) = module_weak.upgrade() { container.set_visible(true); }

        for (i, clock) in clocks.iter().enumerate() {
            if i >= rows.len() { break; }
            let tz_time = std::process::Command::new("date")
                .env("TZ", &clock.zone)
                .arg("+%H:%M")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            rows[i].set_label(&format!("{} - {}", clock.name, tz_time));
        }
        glib::ControlFlow::Continue
    });

    module.container
}
