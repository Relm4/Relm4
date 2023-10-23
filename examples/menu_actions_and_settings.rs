use gtk::{gio, glib, prelude::*};
use relm4::{gtk, safe_settings_and_actions::extensions::*, RelmWidgetExt};

// This example includes the file: relm4.example.gschema.xml
// More info: https://docs.gtk.org/gio/class.Settings.html

// This macro only ensures settings and actions, but not action groups.
// For custom groups you can define constant strings and place them near it.
relm4::safe_settings_and_actions! {
    // This action safety can be keyboard accelerated because it has no @value and no variants.
    Greeting(group: "app", name: "greeting");

    #[derive(Debug)]
    @value(param: &'a str, map: <str>)
    // This action safety can be keyboard accelerated because it has variants.
    MySetting(group: "app", name: "my-setting") {
        A = ("Option A"),
        B = ("Option B"), // String literals must always be enclosed in \
        C = ("Option C"), // parens for accelerators and settings to work.
    }

    @value(param: i32)
    // I think the accelerators only work with i32 (I guess the settings too):
    ManuallySet(group: "app", name: "manually-set") { A = 0, B = 1, C = 2 }

    @value(param: u8)
    // This action cannot be accelerated because it has @value and no variants.
    Increment(group: "win", name: "increment");

    #[derive(Debug)]
    @value(param: i32) // Let's use this action as a message.
    Decrement(group: "win", name: "decrement") { One = 1, Three = 3, Five = 5 }

    // Stateful examples:

    @value(param: u8)
    @state(param: (&'a str, u8, &'a [u8]), owned: (String, u8, Vec<u8>))
    StatefulWithValue(group: "win", name: "stateful-target");

    @state(param: &'a str, owned: String)
    StatefulWithoutValue(group: "win", name: "stateful-no-target");

    // More possibilities:

    @value(param: Vec<String>)
    Vector(group: "win", name: "vector");

    @value(param: &'a [&'a str], owned: Vec<String>)
    Array(group: "win", name: "array");

    @value(param: &'a [&'a str], map: <array_iter_str> glib::VariantStrIter<'a>)
    StringList(group: "win", name: "string-list");

    @value(param: Vec<(u8, String)>)
    VectorTuple(group: "win", name: "vector_tuple");

    @value(param: &'a [(u8, &'a str)], owned: Vec<(u8, String)>)
    ArrayTuple(group: "win", name: "array_tuple");

    @value(param: (i32, &'a str, (i32, &'a str), &'a [i32]),
           owned: (i32, String , (i32, String ),  Vec<i32>))
    Tuple(group: "win", name: "tuple");

    // Curiously, action safeties whose @value is an array or a tuple,
    // both of copyable primitive type elements, can have variants:

    #[derive(Debug)]
    @value(param: &'a [i32], map: <fixed_array>)
    ArrayEnum(group: "win", name: "array_enum") {
         First = [10, 20],
        Second = [30, 40, 50],
    }

    #[derive(Debug)]
    @value(param: (i32, (i32, i32), i32)) // Tuples can have inner tuples.
    TupleEnum(group: "win", name: "tuple_enum") {
         First = (10, (20, 30), 40),
        Second = (1, (2, 3), 4),
    }

    // Settings for saving widget states:

    @state(param: i32)
    // For settings it is not required to specify the group parameter.
    WindowWidth(name: "window-width");

    @state(param: i32)
    // Visibility modifiers supported:
    pub(crate) WindowHeight(name: "window-height");

    @state(param: bool) // For settings without variants, @state is used.
    Toggle(group: "win", name: "toggle");

    #[derive(Debug)]
    @value(param: &'a str, map: <str>)
    // @state is unnecessary because the "value of the key becomes the state of the action":
    // https://docs.gtk.org/gio/method.Settings.create_action.html
    View(group: "win", name: "last-view") {
         First = ("first"),
        Second = ("second"), // And variants require @value.
         Third = ("third"),
    }
}

#[derive(Debug)]
enum Msg {
    Increment(u8),
    Decrement(Decrement),
}

struct Model {
    counter: u8,
}

#[relm4::component]
impl relm4::SimpleComponent for Model {
    type Init = gtk::Application;
    type Input = Msg;
    type Output = ();

    view! {
        #[local]
        app -> gtk::Application {
            add_action = &gio::SimpleAction::new_safe::<Greeting>() {
                connect_activate_safe[label] => move |Greeting, _| label.set_label("Hello world!"),
            },

            add_action = &settings.create_action_safe::<MySetting>() {
                connect_state_notify_safe_enum[label] => move |_, target: MySetting|
                    label.set_label(&format!("Selected {target:?}")),
            },

            add_action = &gio::SimpleAction::new_safe::<ManuallySet>() {
                connect_activate_safe_enum[settings] => move |_, target| match target {
                    ManuallySet::A => settings.set_safe_enum(MySetting::A),
                    ManuallySet::B => settings.set_safe_enum(MySetting::B),
                    ManuallySet::C => settings.set_safe_enum(MySetting::C),
                }.unwrap_or_else(move |error| println!("Failed to save {target} setting: {error}")),
            },

            set_accels_for_action_safe[&["<Ctrl>G"]]: Greeting,

            set_accels_for_action_safe[&["<Ctrl>A"]]: MySetting::A,
            set_accels_for_action_safe[&["<Ctrl>B"]]: MySetting::B,
            set_accels_for_action_safe[&["<Ctrl>C"]]: MySetting::C,

            set_accels_for_action_safe[&["<Shift>A"]]: ManuallySet::A,
            set_accels_for_action_safe[&["<Shift>B"]]: ManuallySet::B,
            set_accels_for_action_safe[&["<Shift>C"]]: ManuallySet::C,

            set_accels_for_action_safe[&["<Ctrl>1"]]: Decrement::One,
            set_accels_for_action_safe[&["<Ctrl>3"]]: Decrement::Three,
            set_accels_for_action_safe[&["<Ctrl>5"]]: Decrement::Five,

            set_accels_for_action_safe[&["<Alt>A"]]: ArrayEnum::First,
            set_accels_for_action_safe[&["<Alt>S"]]: ArrayEnum::Second,

            set_accels_for_action_safe[&["<Alt>D"]]: TupleEnum::First,
            set_accels_for_action_safe[&["<Alt>F"]]: TupleEnum::Second,

            set_accels_for_action_safe[&["<Ctrl>D"]]: StatefulWithoutValue,
        },
        #[root]
        gtk::ApplicationWindow {
            set_title: Some("Actions Example"),
            set_default_width: settings.get_safe(WindowWidth),
            set_default_height: settings.get_safe(WindowHeight),

            connect_close_request[settings] => move |this| {
                settings.set_safe(WindowWidth, this.default_width()).unwrap_or_else(
                    move |error| println!("Failed to save {WindowWidth} setting: {error}")
                );
                settings.set_safe(WindowHeight, this.default_height()).unwrap_or_else(
                    move |error| println!("Failed to save {WindowHeight} setting: {error}")
                );
                glib::Propagation::Stop
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 6,
                set_spacing: 6,

                gtk::StackSwitcher { set_stack: Some(&view) },

                gtk::Box {
                    set_spacing: 6,

                    #[name(toggle)]
                    gtk::CheckButton {
                        #[watch]
                        set_label: Some(&format!("Counter: {}", model.counter)),
                    },

                    #[name(label)]
                    gtk::Label {
                        set_hexpand: true,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_label: &format!("Last setting: {:?}", settings.get_safe_enum::<MySetting>()),
                    },
                },

                #[name(view)]
                gtk::Stack {
                    add_child = &gtk::Button {
                        set_label: "Increment counter by 2",
                        set_target_safe: (Increment, 2),
                        set_hexpand: true,
                    } -> {
                        set_title: "First",
                        set_name: View::First.value(),
                    },

                    add_child = &gtk::Box {
                        set_spacing: 6,

                        gtk::Button {
                            set_label: "Greet",
                            set_action_safe: Greeting,
                            set_hexpand: true,
                        },
                        gtk::Button {
                            set_label: "Stateful With Target (+1)",
                            set_target_safe: (StatefulWithValue, 1),
                            set_hexpand: true,
                        },
                    } -> {
                        set_title: "Second",
                        set_name: View::Second.value(),
                    },

                    add_named[View::Third.some()] = &gtk::MenuButton {
                        set_icon_name: "open-menu",

                        #[wrap(Some)]
                        set_menu_model = &gio::Menu {
                            action["Greet"]: Greeting,

                            submenu["Settings"] = &gio::Menu {
                                section["My Setting"] = &gio::Menu {
                                    action["Option A"]: MySetting::A,
                                    action["Option B"]: MySetting::B,
                                    action["Option C"]: MySetting::C,
                                },

                                section["Manually Set"] = &gio::Menu {
                                    action["Option A"]: ManuallySet::A,
                                    action["Option B"]: ManuallySet::B,
                                    action["Option C"]: ManuallySet::C,
                                },
                            },

                            target["Increment Counter by 10"]: (Increment, 10),

                            submenu["More Actions"] = &gio::Menu {
                                target["Vector"]: (Vector, vec!["one".into(), "two".into()]),
                                target["Array"]: (Array, &["three", "four"]),
                                target["String List"]: (Array, &["five", "six"]),
                                target["Vector Tuple"]: (VectorTuple, vec![(1, "one".into()), (2, "two".into())]),
                                target["Array Tuple"]: (ArrayTuple, &[(3, "three"), (4, "four")]),
                                target["Tuple"]: (Tuple, (1, "hello", (2, "world"), &[3, 4, 5])),

                                section[""] = &gio::Menu {
                                    action["Array Enum First"]: ArrayEnum::First,
                                    action["Array Enum Second"]: ArrayEnum::Second,
                                },

                                section[""] = &gio::Menu {
                                    action["Tuple Enum First"]: TupleEnum::First,
                                    action["Tuple Enum Second"]: TupleEnum::Second,
                                },
                            },

                            section["Decrement Counter"] = &gio::Menu {
                                action["By One"  ]: Decrement::One,
                                action["By Three"]: Decrement::Three,
                                action["By Five" ]: Decrement::Five,
                            },

                            section["Stateful"] = &gio::Menu {
                                action["Without Target"]: StatefulWithoutValue,
                                target["With Target (+10)"]: (StatefulWithValue, 10),
                            },

                            freeze: (),
                        },
                    } -> { set_title: "Third" },
                },
            },
            add_action = &gio::SimpleAction::new_safe::<Increment>() {
                connect_activate_safe_with_target[sender] =>
                    move |Increment, _, target| sender.input(Msg::Increment(target)),
            },
            add_action = &gio::SimpleAction::new_safe::<Decrement>() {
                connect_activate_safe_enum[sender] =>
                    move |_, target| sender.input(Msg::Decrement(target)),
            },
            add_action = &settings.create_action_safe::<View>() {
                connect_state_notify_safe_enum[label] => move |_, target: View|
                    label.set_label(&format!("Now the view is: {target:?}")),
            },
            add_action = &settings.create_action_safe::<Toggle>() {
                connect_state_notify_safe[label] => move |Toggle, _, target|
                    label.set_label(&format!("Toggle changed to: {target:?}")),
            },
            add_action = &gio::SimpleAction::new_safe::<Vector>() {
                connect_activate_safe_with_target[label] => move |Vector, _, target|
                    label.set_label(&format!("{target:?}")),
            },
            add_action = &gio::SimpleAction::new_safe::<Array>() {
                connect_activate_safe_with_target[label] => move |Array, _, target|
                    label.set_label(&format!("{target:?}")),
            },
            add_action = &gio::SimpleAction::new_safe::<StringList>() {
                connect_activate_safe_with_target[label] => move |StringList, _, target|
                    label.set_label(&format!("{target:?}")),
            },
            add_action = &gio::SimpleAction::new_safe::<VectorTuple>() {
                connect_activate_safe_with_target[label] => move |VectorTuple, _, target|
                    label.set_label(&format!("{target:?}")),
            },
            add_action = &gio::SimpleAction::new_safe::<ArrayTuple>() {
                connect_activate_safe_with_target[label] => move |ArrayTuple, _, target|
                    label.set_label(&format!("{target:?}")),
            },
            add_action = &gio::SimpleAction::new_safe::<Tuple>() {
                connect_activate_safe_with_target[label] => move |Tuple, _, target|
                    label.set_label(&format!("{target:?}")),
            },
            add_action = &gio::SimpleAction::new_safe::<ArrayEnum>() {
                connect_activate_safe_enum[label] => move |_, target: ArrayEnum|
                    label.set_label(&(format!("{target:?} = {:?}", target.value()))),
            },
            add_action = &gio::SimpleAction::new_safe::<TupleEnum>() {
                connect_activate_safe_enum[label] => move |_, target: TupleEnum|
                    label.set_label(&(format!("{target:?} = {:?}", target.value()))),
            },
            add_action = &gio::SimpleAction::new_stateful_safe::<StatefulWithoutValue>(" :D ") {
                connect_activate_safe_with_mut_state =>
                    move |StatefulWithoutValue, _this, state| state.push_str(" :D "),

                connect_activate_safe_with_state[label] =>
                    move |StatefulWithoutValue, _this, state| label.set_label(&format!("{state:?}")),
            },
            add_action = &gio::SimpleAction::new_stateful_safe::<StatefulWithValue>(("#", 5, &[0, 1])) {
                connect_activate_safe_with_target_and_mut_state =>
                    move |StatefulWithValue, _this, target, (string, number, vec)| {
                        string.push('_');
                        *number = number.wrapping_add(target);
                        vec.push(vec.len() as _);
                    },

                connect_activate_safe_with_target_and_state[label] =>
                    move |StatefulWithValue, _this, _target, state|
                        label.set_label(&format!("{state:?}")),
            },
        }
    }

    fn init(
        app: gtk::Application,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let settings = gio::Settings::new("relm4.example");
        let model = Model { counter: 0 };
        let widgets = view_output!();

        settings
            .bind_safe::<View>(&widgets.view, "visible-child-name")
            .build();
        settings
            .bind_safe::<Toggle>(&widgets.toggle, "active")
            .build();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Msg, _sender: relm4::ComponentSender<Self>) {
        match msg {
            Msg::Increment(target) => {
                self.counter = self.counter.wrapping_add(target);
            }
            Msg::Decrement(target) => {
                self.counter = self.counter.wrapping_sub(target.value() as u8)
            }
        }
    }
}

fn main() {
    let app = gtk::Application::default();
    relm4::RelmApp::from_app(app.clone()).run::<Model>(app);
}
