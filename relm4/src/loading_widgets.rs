//! Utilities for removing temporary widgets from
//! async factories or components.

use crate::RelmRemoveExt;

trait RemoveTempChild {
    fn remove(&mut self);
}

struct TempWidgetsInner<C: RelmRemoveExt> {
    container: C,
    children: Vec<C::Child>,
}

impl<C: RelmRemoveExt> RemoveTempChild for TempWidgetsInner<C>
where
    C::Child: AsRef<C::Child>,
{
    fn remove(&mut self) {
        for child in &mut self.children {
            self.container.container_remove(&child);
        }
    }
}

/// A type that stores widget containers and their child
/// widgets and removes all children automatically when dropped.
///
/// This mechanism is used by async components and factories
/// to show widgets while the async init function isn't completed.
/// Once the actual widgets are initialized, the temporary loading
/// widgets can be removed again, which is simply done with this type.
pub struct LoadingWidgets {
    containers: Vec<Box<dyn RemoveTempChild>>,
}

impl std::fmt::Debug for LoadingWidgets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadingWidgets")
            .field("containers", &self.containers.len())
            .finish()
    }
}

impl Drop for LoadingWidgets {
    fn drop(&mut self) {
        for child in &mut self.containers {
            child.remove();
        }
    }
}

impl LoadingWidgets {
    fn temp_child<C, W>(container: &C, children: &[W]) -> Box<dyn RemoveTempChild>
    where
        C: RelmRemoveExt + Clone + 'static,
        W: AsRef<C::Child>,
        C::Child: Clone + AsRef<C::Child>,
    {
        let container = container.clone();
        let children = children.iter().map(|c| c.as_ref().clone()).collect();
        let temp_child: TempWidgetsInner<C> = TempWidgetsInner {
            container,
            children,
        };

        Box::new(temp_child)
    }

    /// Create new [`LoadingWidgets`] with one child.
    pub fn new<C, W>(container: &C, child: W) -> Self
    where
        C: RelmRemoveExt + Clone + 'static,
        W: AsRef<C::Child>,
        C::Child: Clone + AsRef<C::Child>,
    {
        Self::with_children(container, &[child])
    }

    /// Create new [`LoadingWidgets`] with multiple children.
    pub fn with_children<C, W>(container: &C, children: &[W]) -> Self
    where
        C: RelmRemoveExt + Clone + 'static,
        W: AsRef<C::Child>,
        C::Child: Clone + AsRef<C::Child>,
    {
        let temp_child = Self::temp_child(container, children);
        Self {
            containers: vec![temp_child],
        }
    }

    /// Add another child to the temporary loading widgets.
    pub fn push<C, W>(&mut self, container: &C, child: W)
    where
        C: RelmRemoveExt + Clone + 'static,
        W: AsRef<C::Child>,
        C::Child: Clone + AsRef<C::Child>,
    {
        let temp_child = Self::temp_child(container, &[child]);
        self.containers.push(temp_child);
    }

    /// Add many children to the temporary loading widgets.
    pub fn add_many<C, W>(&mut self, container: &C, children: &[W])
    where
        C: RelmRemoveExt + Clone + 'static,
        W: AsRef<C::Child>,
        C::Child: Clone + AsRef<C::Child>,
    {
        let temp_child = Self::temp_child(container, children);
        self.containers.push(temp_child);
    }
}
