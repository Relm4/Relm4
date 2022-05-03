mod gtk;

#[cfg(feature = "libadwaita")]
#[cfg_attr(doc, doc(cfg(feature = "libadwaita")))]
mod libadwaita;

#[cfg(feature = "libpanel")]
#[cfg_attr(doc, doc(cfg(feature = "libpanel")))]
mod libpanel;

#[cfg(test)]
mod tests;
