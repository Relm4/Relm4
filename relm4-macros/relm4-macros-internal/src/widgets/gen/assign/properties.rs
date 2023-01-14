use syn::Ident;

use crate::widgets::Properties;

use super::AssignInfo;

impl Properties {
    pub(super) fn assign_stream<'a>(&'a self, info: &mut AssignInfo<'a>, sender_name: &'a Ident) {
        for prop in &self.properties {
            prop.assign_stream(info, sender_name);
        }
    }
}
