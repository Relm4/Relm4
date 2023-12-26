use gtk::glib;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, Cast, GtkApplicationExt, IsA, WidgetExt};
use std::fmt::Debug;
use std::rc::Rc;

use crate::component::{AsyncComponent, AsyncComponentBuilder, AsyncComponentController};
use crate::runtime_util::shutdown_all;
use crate::{Component, ComponentBuilder, ComponentController, MessageBroker, RUNTIME};

use std::cell::{Cell, RefCell};

/// An app that runs the main application.
#[derive(Debug)]
pub struct RelmApp<M: Debug + 'static> {
    /// The [`gtk::Application`] that's used internally to setup
    /// and run your application.
    app: gtk::Application,
    broker: Option<&'static MessageBroker<M>>,
    args: Option<Vec<String>>,
    /// If `true`, make the window visible on
    /// every activation.
    visible: bool,
}

impl<M: Debug + 'static> RelmApp<M> {
    /// Create a new Relm4 application.
    ///
    /// This function will create a new [`gtk::Application`] object if necessary.
    ///
    /// If the `libadwaita` feature is enabled, then the created [`gtk::Application`] will be an
    /// instance of [`adw::Application`]. This can be overridden by passing your own application
    /// object to [`RelmApp::from_app`].
    #[must_use]
    pub fn new(app_id: &str) -> Self {
        crate::init();
        let app = crate::main_application();
        app.set_application_id(Some(app_id));

        Self {
            app,
            broker: None,
            args: None,
            visible: true,
        }
    }

    /// Create a Relm4 application with a provided [`gtk::Application`].
    pub fn from_app(app: impl IsA<gtk::Application>) -> Self {
        let app = app.upcast();
        crate::set_main_application(app.clone());

        Self {
            app,
            broker: None,
            args: None,
            visible: true,
        }
    }

    /// Add [`MessageBroker`] to the top-level component.
    #[must_use]
    pub fn with_broker(mut self, broker: &'static MessageBroker<M>) -> Self {
        self.broker = Some(broker);
        self
    }

    /// Add command line arguments to run with.
    #[must_use]
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = Some(args);
        self
    }

    /// If `true`, make the window visible whenever
    /// the app is activated (e. g. every time [`RelmApp::run`] is called).
    ///
    /// By default, this value is `true`.
    /// If you don't want the window to be visible immediately
    /// (especially when using async components), you can set this
    /// to `false` and call [`WidgetExt::set_visible()`] manually
    /// on your window.
    #[must_use]
    pub fn visible_on_activate(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets a custom global stylesheet.
    pub fn set_global_css(&self, style_data: &str) {
        let display = gtk::gdk::Display::default().unwrap();
        let provider = gtk::CssProvider::new();
        provider.load_from_data(style_data);

        #[allow(deprecated)]
        gtk::StyleContext::add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    /// Sets a custom global stylesheet from a file.
    ///
    /// If the file doesn't exist a [`tracing::error`] message will be emitted and
    /// an [`std::io::Error`] will be returned.
    pub fn set_global_css_from_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), std::io::Error> {
        std::fs::read_to_string(path)
            .map(|bytes| self.set_global_css(&bytes))
            .map_err(|err| {
                tracing::error!("Couldn't load global CSS from file: {}", err);
                err
            })
    }

    /// Runs the application, returns once the application is closed.
    pub fn run<C>(self, payload: C::Init)
    where
        C: Component<Input = M>,
        C::Root: AsRef<gtk::Window>,
    {
        let Self {
            app,
            broker,
            args,
            visible,
        } = self;

        let payload = Cell::new(Some(payload));

        app.connect_startup(move |app| {
            if let Some(payload) = payload.take() {
                let builder = ComponentBuilder::<C>::default();

                let connector = match broker {
                    Some(broker) => builder.launch_with_broker(payload, broker),
                    None => builder.launch(payload),
                };

                // Run late initialization for transient windows for example.
                crate::late_initialization::run_late_init();

                let mut controller = connector.detach();
                let window = controller.widget();
                app.add_window(window.as_ref());

                controller.detach_runtime();
            }
        });

        app.connect_activate(move |app| {
            if let Some(window) = app.active_window() {
                if visible {
                    window.set_visible(true);
                }
            }
        });

        let _guard = RUNTIME.enter();
        if let Some(args) = args {
            app.run_with_args(&args);
        } else {
            app.run();
        }

        // Make sure everything is shut down
        shutdown_all();
        glib::MainContext::ref_thread_default().iteration(true);
    }

    /// Runs the application, returns once the application is closed.
    pub fn run_application<C>(self, payload: C::Init)
    where
        C: Component<Input = M>,
        C::Root: AsRef<gtk::Application>,
    {
        let Self {
            app,
            broker,
            args,
            ..
        } = self;

        let hold_guard = Rc::new(RefCell::new(Some(app.hold())));
        app.connect_window_added(move |_app, _window| {
            hold_guard.borrow_mut().take();
        });

        let payload = Cell::new(Some(payload));
        app.connect_startup(move |_app| {
            if let Some(payload) = payload.take() {
                let builder = ComponentBuilder::<C>::default();
                let connector = match broker {
                    Some(broker) => builder.launch_with_broker(payload, broker),
                    None => builder.launch(payload),
                };

                // Run late initialization for transient windows for example.
                crate::late_initialization::run_late_init();

                let mut controller = connector.detach();
                controller.detach_runtime();
            }
        });


        let _guard = RUNTIME.enter();
        if let Some(args) = args {
            app.run_with_args(&args);
        } else {
            app.run();
        }

        // Make sure everything is shut down
        shutdown_all();
        glib::MainContext::ref_thread_default().iteration(true);
    }


    /// Runs the application, returns once the application is closed.
    pub fn run_async<C>(self, payload: C::Init)
    where
        C: AsyncComponent<Input = M>,
        C::Root: AsRef<gtk::Window>,
    {
        let Self {
            app,
            broker,
            args,
            visible: set_visible,
        } = self;

        let payload = Cell::new(Some(payload));
        app.connect_startup(move |app| {
            if let Some(payload) = payload.take() {
                let builder = AsyncComponentBuilder::<C>::default();

                let connector = match broker {
                    Some(broker) => builder.launch_with_broker(payload, broker),
                    None => builder.launch(payload),
                };

                // Run late initialization for transient windows for example.
                crate::late_initialization::run_late_init();

                let mut controller = connector.detach();
                let window = controller.widget();
                app.add_window(window.as_ref());

                controller.detach_runtime();
            }
        });

        app.connect_activate(move |app| {
            if let Some(window) = app.active_window() {
                if set_visible {
                    window.set_visible(true);
                }
            }
        });

        let _guard = RUNTIME.enter();
        if let Some(args) = args {
            app.run_with_args(&args);
        } else {
            app.run();
        }

        // Make sure everything is shut down
        shutdown_all();
        glib::MainContext::ref_thread_default().iteration(true);
    }

    /// Runs the application, returns once the application is closed.
    pub fn run_application_async<C>(self, payload: C::Init)
    where
        C: AsyncComponent<Input = M>,
        C::Root: AsRef<gtk::Application>,
    {
        let Self {
            app,
            broker,
            args,
            ..
        } = self;

        let hold_guard = Rc::new(RefCell::new(Some(app.hold())));
        app.connect_window_added(move |_app, _window| {
            hold_guard.borrow_mut().take();
        });

        let payload = Cell::new(Some(payload));
        app.connect_startup(move |_app| {
            if let Some(payload) = payload.take() {
                let builder = AsyncComponentBuilder::<C>::default();
                let connector = match broker {
                    Some(broker) => builder.launch_with_broker(payload, broker),
                    None => builder.launch(payload),
                };

                // Run late initialization for transient windows for example.
                crate::late_initialization::run_late_init();

                let mut controller = connector.detach();
                controller.detach_runtime();
            }
        });


        let _guard = RUNTIME.enter();
        if let Some(args) = args {
            app.run_with_args(&args);
        } else {
            app.run();
        }


        // Make sure everything is shut down
        shutdown_all();
        glib::MainContext::ref_thread_default().iteration(true);
    }

}
