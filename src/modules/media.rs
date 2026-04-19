use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

fn read_mpris() -> (String, String, String, bool) {
    let out = run_cmd(&[
        "playerctl", "--format",
        "{{title}}|||{{artist}}|||{{mpris:artUrl}}|||{{status}}",
        "metadata",
    ]);
    if out.is_empty() {
        return (String::new(), String::new(), String::new(), false);
    }
    let parts: Vec<&str> = out.splitn(4, "|||").collect();
    let title = parts.get(0).copied().unwrap_or("").trim().to_string();
    let artist = parts.get(1).copied().unwrap_or("").trim().to_string();
    let art_url = parts.get(2).copied().unwrap_or("").trim().to_string();
    let status = parts.get(3).copied().unwrap_or("").trim().to_string();
    let playing = status.to_lowercase() == "playing";
    (title, artist, art_url, playing)
}

pub fn new_music() -> gtk4::Box {
    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    box_.set_widget_name("custom-music");
    box_.set_visible(false);

    let cover_shell = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    cover_shell.set_widget_name("custom-music-cover");

    let cover = gtk4::Picture::new();
    cover.set_can_shrink(true);
    cover.set_content_fit(gtk4::ContentFit::Cover);
    cover.set_size_request(22, 22);

    let play_icon = gtk4::Image::from_icon_name("media-playback-start-symbolic");
    play_icon.set_margin_end(4);
    play_icon.set_pixel_size(14);

    cover_shell.append(&cover);
    cover_shell.append(&play_icon);

    let text_shell = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    text_shell.set_widget_name("custom-music-text");

    let label = gtk4::Label::new(None);
    label.set_single_line_mode(true);
    label.set_ellipsize(pango::EllipsizeMode::End);
    label.set_xalign(0.0);
    label.set_hexpand(false);
    text_shell.append(&label);

    box_.append(&cover_shell);
    box_.append(&text_shell);

    let (popup_music, menu) = make_popup();
    menu.set_widget_name("music-menu");
    menu.set_spacing(6);
    menu.set_size_request(240, -1);

    let popup_title = gtk4::Label::new(None);
    popup_title.add_css_class("music-title");
    popup_title.set_xalign(0.0);
    menu.append(&popup_title);

    let popup_artist = gtk4::Label::new(None);
    popup_artist.add_css_class("music-artist");
    popup_artist.set_xalign(0.0);
    menu.append(&popup_artist);

    let controls = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    controls.set_halign(gtk4::Align::Center);

    let btn_prev = gtk4::Button::from_icon_name("media-skip-backward-symbolic");
    btn_prev.set_has_frame(false);
    let btn_play = gtk4::Button::from_icon_name("media-playback-pause-symbolic");
    btn_play.set_has_frame(false);
    let btn_next = gtk4::Button::from_icon_name("media-skip-forward-symbolic");
    btn_next.set_has_frame(false);

    controls.append(&btn_prev);
    controls.append(&btn_play);
    controls.append(&btn_next);
    menu.append(&controls);

    btn_prev.connect_clicked(|_| { run_detached_args("playerctl", &["previous"]); });
    btn_play.connect_clicked(|_| { run_detached_args("playerctl", &["play-pause"]); });
    btn_next.connect_clicked(|_| { run_detached_args("playerctl", &["next"]); });

    attach_hover_popup(&box_, &popup_music, || {});

    let step = std::rc::Rc::new(std::cell::Cell::new(0usize));
    let box_weak = box_.downgrade();
    let label_weak = label.downgrade();
    let cover_weak = cover.downgrade();
    let play_icon_weak = play_icon.downgrade();
    let popup_title_weak = popup_title.downgrade();
    let popup_artist_weak = popup_artist.downgrade();

    let update = {
        let step = step.clone();
        let box_weak = box_weak.clone();
        let label_weak = label_weak.clone();
        let cover_weak = cover_weak.clone();
        let play_icon_weak = play_icon_weak.clone();
        let popup_title_weak = popup_title_weak.clone();
        let popup_artist_weak = popup_artist_weak.clone();

        std::rc::Rc::new(move || {
            let (title, artist, art_url, playing) = read_mpris();
            let box_ = match box_weak.upgrade() { Some(b) => b, None => return };
            let label = match label_weak.upgrade() { Some(l) => l, None => return };
            let cover = match cover_weak.upgrade() { Some(c) => c, None => return };
            let play_icon = match play_icon_weak.upgrade() { Some(i) => i, None => return };

            if title.is_empty() && artist.is_empty() {
                box_.set_visible(false);
                return;
            }
            box_.set_visible(true);

            let full_text = if artist.is_empty() { title.clone() } else { format!("{} - {}", title, artist) };
            let s = step.get();
            let display = marquee_text(&full_text, 30, s);
            step.set(s + 1);
            label.set_label(&display);

            let icon_name = if playing { "media-playback-pause-symbolic" } else { "media-playback-start-symbolic" };
            play_icon.set_icon_name(Some(icon_name));

            if !art_url.is_empty() {
                let file = gio::File::for_uri(&art_url);
                cover.set_file(Some(&file));
            } else {
                cover.set_paintable(None::<&gdk4::Paintable>);
            }

            if let Some(pt) = popup_title_weak.upgrade() { pt.set_label(&title); }
            if let Some(pa) = popup_artist_weak.upgrade() { pa.set_label(&artist); }
        })
    };

    (update)();

    let update2 = update.clone();
    glib::timeout_add_local(Duration::from_millis(280), move || {
        if box_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        (update2)();
        glib::ControlFlow::Continue
    });

    let rx = super::events::mpris_event_channel();
    let update3 = update.clone();
    glib::MainContext::default().spawn_local(async move {
        while rx.recv().await.is_ok() {
            (update3)();
        }
    });

    box_
}

pub fn new_mpd() -> gtk4::Box {
    let module = TextModule::new("custom-mpd");
    let box_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();

    glib::timeout_add_local(Duration::from_secs(3), move || {
        if box_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        let current = run_cmd(&["mpc", "current"]);
        let status = run_cmd(&["mpc", "status"]);
        let text = if current.is_empty() {
            String::new()
        } else {
            let playing = !status.contains("[paused]");
            let icon = if playing { "" } else { "" };
            format!("{} {}", icon, truncate(&current, 30))
        };
        if let Some(l) = label_weak.upgrade() { l.set_label(&text); }
        if let Some(b) = box_weak.upgrade() { b.set_visible(!text.is_empty()); }
        glib::ControlFlow::Continue
    });

    module.container
}
