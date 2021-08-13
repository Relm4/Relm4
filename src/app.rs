/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use gtk::glib;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};

use std::marker::PhantomData;

use crate::{AppUpdate, Components, Model as ModelTrait, Widgets as WidgetsTrait};

/// The app that runs the main application.
/// A [`RelmApp`] consists of a model that stores the application state
/// and widgets that represent the UI.
///
/// A [`RelmApp`] might run as a standalone application or may contain
/// multiple components that communicate with each other.
pub struct RelmApp<Model>
where
    Model: ModelTrait + AppUpdate + 'static,
    Model::Widgets: WidgetsTrait<Model, (), Root = gtk::ApplicationWindow> + 'static,
    Model::Components: Components<Model> + 'static,
{
    model: PhantomData<Model>,
    app: gtk::Application,
}

impl<Model> RelmApp<Model>
where
    Model: ModelTrait + AppUpdate + 'static,
    Model::Widgets: WidgetsTrait<Model, (), Root = gtk::ApplicationWindow> + 'static,
    Model::Components: Components<Model> + 'static,
{
    /// Runs the application, returns once the application is closed.
    pub fn run(&self) {
        self.app.run();
    }

    /// Create an application.
    pub fn new(mut model: Model) -> Self {
        gtk::init().expect("Couln't initialize GTK");
        let app = gtk::Application::builder().build();
        crate::APP
            .set(fragile::Fragile::new(app.clone()))
            .expect("APP was alredy set");

        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut widgets = Model::Widgets::init_view(&model, &(), sender.clone());
        let root = widgets.root_widget();

        let components = Model::Components::init_components(&model, &widgets, sender.clone());

        widgets.connect_components(&components);

        // Initialize GTK
        app.connect_activate(move |app| {
            root.set_application(Some(app));
            root.present();
        });

        {
            let context = glib::MainContext::default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");

            let app = app.clone();

            // Register receiver on the main loop to wait for messages to update model and view the changes.
            receiver.attach(Some(&context), move |msg: Model::Msg| {
                if !model.update(msg, &components, sender.clone()) {
                    app.quit();
                    return glib::Continue(false);
                }
                widgets.view(&model, sender.clone());
                glib::Continue(true)
            });
        }

        RelmApp {
            model: PhantomData,
            app,
        }
    }
}
