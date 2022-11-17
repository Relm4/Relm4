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
            //let adaptor: ChildRefAdaptor<C> = ChildRefAdaptor { child };
            self.container.container_remove(&child);
        }
    }
}

pub struct TempWidgets {
    temp_children: Vec<Box<dyn RemoveTempChild>>,
}

impl TempWidgets {
    fn temp_child<C, W>(container: &C, children: &[W]) -> Box<dyn RemoveTempChild>
    where
        C: RelmRemoveExt + Clone + 'static,
        W: AsRef<C::Child>,
        C::Child: Clone + AsRef<C::Child>,
    {
        let container = container.clone();
        let children = children.into_iter().map(|c| c.as_ref().clone()).collect();
        let temp_child: TempWidgetsInner<C> = TempWidgetsInner {
            container,
            children,
        };

        Box::new(temp_child)
    }

    pub fn remove(self) {
        for mut child in self.temp_children {
            child.remove();
        }
    }

    pub fn new<C, W>(container: &C, child: W) -> Self
    where
        C: RelmRemoveExt + Clone + 'static,
        W: AsRef<C::Child>,
        C::Child: Clone + AsRef<C::Child>,
    {
        Self::with_children(container, &[child])
    }

    pub fn with_children<C, W>(container: &C, children: &[W]) -> Self
    where
        C: RelmRemoveExt + Clone + 'static,
        W: AsRef<C::Child>,
        C::Child: Clone + AsRef<C::Child>,
    {
        let temp_child = Self::temp_child(container, children);
        Self {
            temp_children: vec![temp_child],
        }
    }

    pub fn push<C, W>(&mut self, container: &C, child: W)
    where
        C: RelmRemoveExt + Clone + 'static,
        W: AsRef<C::Child>,
        C::Child: Clone + AsRef<C::Child>,
    {
        let temp_child = Self::temp_child(container, &[child]);
        self.temp_children.push(temp_child);
    }

    pub fn add_many<C, W>(&mut self, container: &C, children: &[W])
    where
        C: RelmRemoveExt + Clone + 'static,
        W: AsRef<C::Child>,
        C::Child: Clone + AsRef<C::Child>,
    {
        let temp_child = Self::temp_child(container, children);
        self.temp_children.push(temp_child);
    }
}
