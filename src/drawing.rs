//! Utility to help drawing on a [`gtk::DrawingArea`] in a Relm4 application.
//! Create a [`DrawHandler`], initialize it, and get its context when handling a message (that could be
//! sent from the draw signal).

// TODO: check if clip has the intended behavior.

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use gtk::cairo::{self, Context, Format, ImageSurface};
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
    draw_area: gtk::DrawingArea,
}

impl DrawContext {
    fn new(
        draw_surface: &Surface,
        edit_surface: &ImageSurface,
        draw_area: &gtk::DrawingArea,
    ) -> Result<Self, cairo::Error> {
        let draw_context = Self {
            context: Context::new(edit_surface)?,
            draw_surface: draw_surface.clone(),
            edit_surface: edit_surface.clone(),
            draw_area: draw_area.clone(),
        };

        Ok(draw_context)
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
        self.draw_area.queue_draw();
    }
}

/// Manager for drawing operations.
#[derive(Debug)]
pub struct DrawHandler {
    draw_surface: Surface,
    edit_surface: ImageSurface,
    draw_area: Option<gtk::DrawingArea>,
}

impl DrawHandler {
    /// Create a new [`DrawHandler`].
    pub fn new() -> Result<Self, cairo::Error> {
        Ok(Self {
            draw_surface: Surface::new(ImageSurface::create(Format::ARgb32, 100, 100)?),
            edit_surface: ImageSurface::create(Format::ARgb32, 100, 100)?,
            draw_area: None,
        })
    }

    /// Get the drawing context to draw on a [`gtk::DrawingArea`].
    /// If the size of the [`gtk::DrawingArea`] changed, the contents of the
    /// surface will be replaced by a new, empty surface.
    pub fn get_context(&mut self) -> Result<DrawContext, cairo::Error> {
        if let Some(ref draw_area) = self.draw_area {
            let allocation = draw_area.allocation();
            let scale = draw_area.scale_factor();
            let width = allocation.width() * scale;
            let height = allocation.height() * scale;

            if (width, height) != (self.edit_surface.width(), self.edit_surface.height()) {
                // TODO: also copy the old small surface to the new bigger one?
                match ImageSurface::create(Format::ARgb32, width, height) {
                    Ok(surface) => {
                        surface.set_device_scale(f64::from(scale), f64::from(scale));
                        self.edit_surface = surface;
                    }
                    Err(error) => log::error!("Cannot resize image surface: {:?}", error),
                }
            }
            DrawContext::new(&self.draw_surface, &self.edit_surface, draw_area)
        } else {
            panic!("Call DrawHandler::init() before DrawHandler::get_context().");
        }
    }

    /// Initialize the draw handler.
    pub fn init(&mut self, draw_area: &gtk::DrawingArea) {
        self.draw_area = Some(draw_area.clone());
        let draw_surface = self.draw_surface.clone();
        draw_area.set_draw_func(move |_, context, _, _| {
            // TODO: only copy the area that was exposed?
            if let Err(error) = context.set_source_surface(&draw_surface.get(), 0.0, 0.0) {
                log::error!("Cannot set source surface: {:?}", error);
            }

            if let Err(error) = context.paint() {
                log::error!("Cannot paint: {:?}", error);
            }
        });
    }
}
