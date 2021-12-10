use adw::traits::ApplicationWindowExt;
use gtk::glib::BindingFlags;
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, ObjectExt, OrientableExt, ToggleButtonExt};

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
    start_page: u8,
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
            set_default_width: 400,
            set_default_height: 240,

            set_content: main_box = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,

                append = &adw::HeaderBar {
                    set_title_widget = Some(&gtk::Label) {
                        set_label: "Relm4 advent calendar",
                    },
                    pack_start: flap_toggle = &gtk::ToggleButton {
                        set_label: "Expand",
                        set_icon_name: "sidebar-show",
                    }
                },
                append: flap = &adw::Flap {
                    set_content: main_view = Some(&gtk::Stack) {
                        factory!(model.calendar_entries)
                    },
                    set_flap = Some(&gtk::StackSidebar) {
                        set_stack: &main_view,
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

        main_view.set_visible_child_name(&model.start_page.to_string());

        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            send!(sender, AppMsg::Update);
        });
    }
}

fn main() {
    let mut calendar_entries = FactoryVec::new();
    let mut start_page = 0;

    // Time since midnight December the 1st.
    let time_since_first_dec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 1638313200;
    let seconds_per_day: i64 = 86400;

    let verses = vec![
        ("Come unto me, all ye that labour and are heavy laden, and I will give you rest. Take my yoke upon you, and learn of me; for I am meek and lowly in heart: and ye shall find rest unto your souls. For my yoke is easy, and my burden is light.", "Matthew 11, 28-30"),
        ("For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. For God sent not his Son into the world to condemn the world; but that the world through him might be saved.", "John 3, 16-17"),
        ("If any man thirst, let him come unto me, and drink. He that believeth on me, as the scripture hath said, out of his belly shall flow rivers of living water.", "John 8, 37-38"),
        ("For the LORD God is a sun and shield: the LORD will give grace and glory: no good thing will he withhold from them that walk uprightly.", "Psalm 84, 11"),
        ("Trust in the LORD with all thine heart; and lean not unto thine own understanding. In all thy ways acknowledge him, and he shall direct thy paths.", "Proverbs 3, 5-6"),
        ("Blessed be the God and Father of our Lord Jesus Christ, who has blessed us with every spiritual blessing in the heavenly places in Christ, just as He chose us in Him before the foundation of the world, that we should be holy and without blame before Him in love.", "Ephesians 1,3-4"),
        ("Jesus saith unto him, I am the way, the truth, and the life: no man cometh unto the Father, but by me.", "John 14, 6"),
        ("Peace I leave with you, my peace I give unto you: not as the world giveth, give I unto you. Let not your heart be troubled, neither let it be afraid.", "John 14, 27"),
        ("But the fruit of the Spirit is love, joy, peace, longsuffering, gentleness, goodness, faith, meekness, temperance: against such there is no law.", "Romans 8, 22-23"),
        ("Then spake Jesus again unto them, saying, I am the light of the world: he that followeth me shall not walk in darkness, but shall have the light of life.", "John 8, 12"),
        ("Rejoice in the Lord alway: and again I say, Rejoice. Let your moderation be known unto all men. The Lord is at hand.", "Philippians 4, 4-5"),
        ("For the word of God is quick, and powerful, and sharper than any twoedged sword, piercing even to the dividing asunder of soul and spirit, and of the joints and marrow, and is a discerner of the thoughts and intents of the heart.", "Hebrews 4, 12"),
        ("V13", "P13"),
        ("V14", "P14"),
        ("V15", "P15"),
        ("V16", "P16"),
        ("V17", "P17"),
        ("V18", "P18"),
        ("V19", "P19"),
        ("V20", "P20"),
        ("V21", "P21"),
        ("V22", "P22"),
        ("V23", "P23"),
        ("V24", "P24"),
    ];

    // Fill factory with the verses
    for (idx, (verse, passage)) in verses.iter().enumerate() {
        let time_difference = seconds_per_day * idx as i64 - time_since_first_dec as i64;
        let time_left = if time_difference > 0 {
            time_difference as u32
        } else {
            start_page = idx as u8;
            0
        };

        calendar_entries.push(CalendarEntry {
            verse: verse.to_string(),
            passage: passage.to_string(),
            time_left,
        });
    }

    let app = RelmApp::new(AppModel {
        calendar_entries,
        start_page,
    });

    // Style the verse labels
    relm4::set_global_css(
        b"\
        .verse { \
            font-weight: bold; \
            font-size: 1.4em; \
        }",
    );
    app.run();
}
