use gtk::glib;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, Cast, GtkApplicationExt, IsA, WidgetExt};
use std::fmt::Debug;

use crate::component::{AsyncComponent, AsyncComponentBuilder, AsyncComponentController};
use crate::runtime_util::shutdown_all;
use crate::{Component, ComponentBuilder, ComponentController, MessageBroker, RUNTIME};

use std::cell::Cell;

/// An app that runs the main application.
#[derive(Debug)]
pub struct RelmApp<M: Debug + 'static> {
    /// The [`gtk::Application`] that's used internally to setup
    /// and run your application.
    app: gtk::Application,
    broker: Option<&'static MessageBroker<M>>,
    args: Option<Vec<String>>,
}

impl<'args, M: Debug + 'static> RelmApp<M> {
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
        }
    }

    pub fn with_broker(mut self, broker: &'static MessageBroker<M>) -> Self {
        self.broker = Some(broker);
        self
    }

    pub fn with_args(mut self, args: &[impl AsRef<str>]) -> Self {
        self.args = Some(args.iter().map(|a| a.as_ref().to_string()).collect());
        self
    }

    /// Runs the application, returns once the application is closed.
    ///
    /// Unlike [`gtk::prelude::ApplicationExtManual::run`], this function
    /// does not handle command-line arguments. To pass arguments to GTK, use
    /// [`RelmApp::run_with_args`].
    pub fn run<C>(self, payload: C::Init)
    where
        C: Component<Input = M>,
        C::Root: IsA<gtk::Window> + WidgetExt,
    {
        let Self { app, broker, args } = self;

        let payload = Cell::new(Some(payload));

        app.connect_activate(move |app| {
            if let Some(payload) = payload.take() {
                assert!(
                    app.is_registered(),
                    "App should be already registered when activated"
                );

                let builder = ComponentBuilder::<C>::default();
                let connector = match broker {
                    Some(broker) => builder.launch_with_broker(payload, broker),
                    None => builder.launch(payload),
                };

                // Run late initialization for transient windows for example.
                crate::late_initialization::run_late_init();

                let mut controller = connector.detach();
                let window = controller.widget().clone();

                controller.detach_runtime();

                app.add_window(window.as_ref());
                window.show();
            }
        });

        let _guard = RUNTIME.enter();
        let args = args.as_ref().map(Vec::as_slice).unwrap_or(&[]);
        app.run_with_args(args);

        // Make sure everything is shut down
        shutdown_all();
        glib::MainContext::ref_thread_default().iteration(true);
    }

    /// Runs the application with the provided command-line arguments, returns once the application
    /// is closed.
    pub fn run_with_args<C, S>(self, payload: C::Init, args: &[S])
    where
        C: Component<Input = M>,
        C::Root: IsA<gtk::Window> + WidgetExt,
        S: AsRef<str>,
    {
        self.with_args(args).run::<C>(payload);
    }

    /// Runs the application, returns once the application is closed.
    ///
    /// Unlike [`gtk::prelude::ApplicationExtManual::run`], this function
    /// does not handle command-line arguments. To pass arguments to GTK, use
    /// [`RelmApp::run_with_args`].
    pub fn run_async<C>(self, payload: C::Init)
    where
        C: AsyncComponent<Input = M>,
        C::Root: IsA<gtk::Window> + WidgetExt,
    {
        let Self { app, broker, args } = self;

        if broker.is_some() {
            panic!("MessageBroker is not implemented for AsyncComponent yet");
        }

        let payload = Cell::new(Some(payload));

        app.connect_activate(move |app| {
            if let Some(payload) = payload.take() {
                assert!(
                    app.is_registered(),
                    "App should be already registered when activated"
                );

                let builder = AsyncComponentBuilder::<C>::default();
                let connector = builder.launch(payload);

                // Run late initialization for transient windows for example.
                crate::late_initialization::run_late_init();

                let mut controller = connector.detach();
                let window = controller.widget().clone();

                controller.detach_runtime();

                app.add_window(window.as_ref());
                window.show();
            }
        });

        let _guard = RUNTIME.enter();
        let args = args.as_ref().map(Vec::as_slice).unwrap_or(&[]);
        app.run_with_args(args);

        // Make sure everything is shut down
        shutdown_all();
        glib::MainContext::ref_thread_default().iteration(true);
    }

    /// Runs the application with the provided command-line arguments, returns once the application
    /// is closed.
    pub fn run_async_with_args<C, S>(self, payload: C::Init, args: &[S])
    where
        C: AsyncComponent<Input = M>,
        C::Root: IsA<gtk::Window> + WidgetExt,
        S: AsRef<str>,
    {
        self.with_args(args).run_async::<C>(payload)
    }
}
