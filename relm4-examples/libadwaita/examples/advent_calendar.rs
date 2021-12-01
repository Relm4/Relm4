use adw::traits::ApplicationWindowExt;
use gtk::glib::BindingFlags;
use gtk::prelude::{
    BoxExt, ButtonExt, GtkWindowExt, ObjectExt, OrientableExt, ToggleButtonExt, WidgetExt,
};

use relm4::{
    factory::{positions::StackPageInfo, FactoryPrototype, FactoryVec},
    send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets,
};

use std::time::{SystemTime, UNIX_EPOCH};

fn waiting_text(time: u32) -> String {
    format!(
        "You need to wait {} seconds before you can open this door.",
        time
    )
}

#[derive(Debug)]
struct CalendarEntry {
    verse: String,
    passage: String,
    time_left: u32,
}

#[derive(Debug)]
struct CalendarWidgets {
    root: gtk::Stack,
    active: gtk::Box,
    waiting: gtk::CenterBox,
    waiting_label: gtk::Label,
}

impl CalendarWidgets {
    // Update widgets to new time
    fn update(&self, time: u32) {
        if time == 0 {
            self.root.set_visible_child(&self.active);
        } else {
            self.waiting_label.set_label(&waiting_text(time));
            self.root.set_visible_child(&self.waiting);
        }
    }
}

impl FactoryPrototype for CalendarEntry {
    type Factory = FactoryVec<CalendarEntry>;
    type Widgets = CalendarWidgets;
    type Root = gtk::Stack;
    type View = gtk::Stack;
    type Msg = AppMsg;

    fn generate(&self, _key: &usize, _sender: Sender<AppMsg>) -> Self::Widgets {
        // Create widgets.
        let root = gtk::Stack::builder()
            .vexpand(true)
            .transition_type(gtk::StackTransitionType::RotateLeftRight)
            .transition_duration(700)
            .build();

        let active = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(7)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .build();
        let verse = gtk::Label::builder()
            .label(self.verse.as_str())
            .css_classes(vec!["verse".to_string()])
            .wrap(true)
            .selectable(true)
            .build();
        let passage = gtk::Label::builder()
            .label(self.passage.as_str())
            .selectable(true)
            .halign(gtk::Align::End)
            .build();

        let waiting = gtk::CenterBox::new();
        let waiting_label = gtk::Label::new(Some(&waiting_text(self.time_left)));

        // Connect widgets.
        active.append(&verse);
        active.append(&passage);
        active.set_margin_all(30);

        waiting.set_center_widget(Some(&waiting_label));

        root.add_child(&active);
        root.add_child(&waiting);

        let widgets = CalendarWidgets {
            root,
            waiting_label,
            waiting,
            active,
        };
        widgets.update(self.time_left);

        widgets
    }

    fn position(&self, key: &usize) -> StackPageInfo {
        StackPageInfo {
            name: Some(key.to_string()),
            title: Some(format!("Day {}", key + 1)),
        }
    }

    fn update(&self, _key: &usize, widgets: &CalendarWidgets) {
        widgets.update(self.time_left);
    }

    fn get_root(widgets: &Self::Widgets) -> &Self::Root {
        &widgets.root
    }
}

struct AppModel {
    calendar_entries: FactoryVec<CalendarEntry>,
}

enum AppMsg {
    Update,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Update => {
                // Check all entries
                for i in 0..self.calendar_entries.len() {
                    // If counter > 1, count down
                    let needs_update = self.calendar_entries.get(i).unwrap().time_left != 0;

                    if needs_update {
                        let entry = self.calendar_entries.get_mut(i).unwrap();
                        entry.time_left = entry.time_left.saturating_sub(1);
                    }
                }
            }
        }
        true
    }
}

/// This hack is needed because ApplicationWindow doesn't implement default.
/// This has already been fixed upstream but wasn't released yet.
fn application_window() -> adw::ApplicationWindow {
    adw::ApplicationWindow::builder().build()
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = application_window() -> adw::ApplicationWindow {
            set_default_width: 300,
            set_default_height: 200,

            set_content: main_box = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,

                append = &adw::HeaderBar {
                    set_title_widget = Some(&gtk::Label) {
                        set_label: "Relm4 advent calendar",
                    },
                    pack_start: flap_toggle = &gtk::ToggleButton {
                        set_label: "Expand"
                    }
                },
                append: flap = &adw::Flap {
                    set_content: main_view = Some(&gtk::Stack) {
                        factory!(model.calendar_entries)
                    },
                    set_flap = Some(&gtk::ScrolledWindow) {
                        set_vexpand: true,
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_child = Some(&gtk::StackSwitcher) {
                            set_stack: Some(&main_view),
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 4,
                        },
                    },
                    set_separator = Some(&gtk::Separator) {}
                }
            },
        }
    }

    // Connect properties and start update thread.
    fn post_init() {
        flap_toggle
            .bind_property("active", &flap, "reveal-flap")
            .flags(BindingFlags::BIDIRECTIONAL)
            .build();

        flap_toggle.set_active(true);

        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            send!(sender, AppMsg::Update);
        });
    }
}

fn main() {
    let mut model = AppModel {
        calendar_entries: FactoryVec::new(),
    };

    // Time since midnight December the 1st.
    let time_since_first_dec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 1638313200;
    let seconds_per_day: i64 = 86400;

    let verses = vec![
        ("Come unto me, all ye that labour and are heavy laden, and I will give you rest. Take my yoke upon you, and learn of me; for I am meek and lowly in heart: and ye shall find rest unto your souls. For my yoke is easy, and my burden is light.", "Matthew 11, 28-30"),
        ("Vers2", "P2"),
        ("V3", "P3"),
        ("V4", "P4"),
        ("V5", "P5"),
        ("V6", "P6"),
        ("V7", "P7"),
        ("V8", "P8"),
    ];

    // Fill factory with the verses
    for (idx, (verse, passage)) in verses.iter().enumerate() {
        let time_difference = seconds_per_day * idx as i64 - time_since_first_dec as i64;
        let time_left = if time_difference > 0 {
            time_difference as u32
        } else {
            0
        };

        model.calendar_entries.push(CalendarEntry {
            verse: verse.to_string(),
            passage: passage.to_string(),
            time_left,
        });
    }

    let app = RelmApp::new(model);

    relm4::set_global_css(
        b"\
        stackswitcher > button { \
             border-radius: 2px; \
             margin-bottom: 3px; \
        } \
        .verse { \
            font-weight: bold; \
            font-size: 1.4em; \
        }",
    );
    app.run();
}
