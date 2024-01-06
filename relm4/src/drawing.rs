//! Utility to help drawing on a [`gtk::DrawingArea`] in a Relm4 application.
//! Create a [`DrawHandler`], initialize it, and get its context when handling a message (that could be
//! sent from the draw signal).

// TODO: check if clip has the intended behavior.

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use gtk::cairo::{Context, Format, ImageSurface};
use gtk::prelude::{DrawingAreaExtManual, WidgetExt};

#[derive(Clone, Debug)]
struct Surface {
    surface: Rc<RefCell<ImageSurface>>,
}

impl Surface {
    fn new(surface: ImageSurface) -> Self {
        Self {
            surface: Rc::new(RefCell::new(surface)),
        }
    }

    fn get(&self) -> ImageSurface {
        self.surface.borrow().clone()
    }

    fn set(&self, surface: &ImageSurface) {
        *self.surface.borrow_mut() = surface.clone();
    }
}

#[derive(Debug)]
/// Context returned by [`DrawHandler`] that stores a [`Context`] along
/// with additional data required for drawing.
pub struct DrawContext {
    context: Context,
    draw_surface: Surface,
    edit_surface: ImageSurface,
    drawing_area: gtk::DrawingArea,
}

impl DrawContext {
    fn new(
        draw_surface: &Surface,
        edit_surface: &ImageSurface,
        drawing_area: &gtk::DrawingArea,
    ) -> Self {
        Self {
            context: Context::new(edit_surface).unwrap(),
            draw_surface: draw_surface.clone(),
            edit_surface: edit_surface.clone(),
            drawing_area: drawing_area.clone(),
        }
    }
}

impl Deref for DrawContext {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl Drop for DrawContext {
    fn drop(&mut self) {
        self.draw_surface.set(&self.edit_surface);
        self.drawing_area.queue_draw();
    }
}

/// Manager for drawing operations.
#[derive(Debug)]
#[must_use]
pub struct DrawHandler {
    draw_surface: Surface,
    edit_surface: ImageSurface,
    drawing_area: gtk::DrawingArea,
}

impl Default for DrawHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawHandler {
    /// Create a new [`DrawHandler`].
    pub fn new() -> Self {
        Self::new_with_drawing_area(gtk::DrawingArea::default())
    }

    /// Create a new [`DrawHandler`] with an existing [`gtk::DrawingArea`].
    pub fn new_with_drawing_area(drawing_area: gtk::DrawingArea) -> Self {
        let draw_surface = Surface::new(ImageSurface::create(Format::ARgb32, 100, 100).unwrap());
        let edit_surface = ImageSurface::create(Format::ARgb32, 100, 100).unwrap();

        use gtk::glib;
        drawing_area.set_draw_func(
            glib::clone!(@strong draw_surface => move |_, context, _, _| {
                // TODO: only copy the area that was exposed?
                if let Err(error) = context.set_source_surface(&draw_surface.get(), 0.0, 0.0) {
                    tracing::error!("Cannot set source surface: {:?}", error);
                }

                if let Err(error) = context.paint() {
                    tracing::error!("Cannot paint: {:?}", error);
                }
            }),
        );

        Self {
            draw_surface,
            edit_surface,
            drawing_area,
        }
    }

    /// Get the drawing context to draw on a [`gtk::DrawingArea`].
    /// If the size of the [`gtk::DrawingArea`] changed, the contents of the
    /// surface will be replaced by a new, empty surface.
    #[allow(deprecated)]
    pub fn get_context(&mut self) -> DrawContext {
        let allocation = self.drawing_area.allocation();
        let scale = self.drawing_area.scale_factor();
        let width = allocation.width() * scale;
        let height = allocation.height() * scale;

        if (width, height) != (self.edit_surface.width(), self.edit_surface.height()) {
            match ImageSurface::create(Format::ARgb32, width, height) {
                Ok(surface) => {
                    surface.set_device_scale(f64::from(scale), f64::from(scale));
                    self.edit_surface = surface;
                }
                Err(error) => tracing::error!("Cannot resize image surface: {:?}", error),
            }
        }
        DrawContext::new(&self.draw_surface, &self.edit_surface, &self.drawing_area)
    }

    /// Get the height of the [`DrawHandler`].
    #[must_use]
    pub fn height(&self) -> i32 {
        self.edit_surface.height()
    }

    /// Get the width of the [`DrawHandler`].
    #[must_use]
    pub fn width(&self) -> i32 {
        self.edit_surface.width()
    }

    /// Get the [`gtk::DrawingArea`] of the [`DrawHandler`].
    #[must_use]
    pub fn drawing_area(&self) -> &gtk::DrawingArea {
        &self.drawing_area
    }
}
