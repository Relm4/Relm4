use crate::RelmObjectExt;

use super::ConnectBinding;

impl ConnectBinding for gtk::ToggleButton {
    type Target = bool;

    fn bind<B: super::Binding<Target = Self::Target>>(&self, binding: &B) {
        self.add_binding(binding, "active")
    }
}
