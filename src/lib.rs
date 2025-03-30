pub mod vec_arrow;

use bevy::app::{Plugin, Update};

/// This plugin adds systems that keep track of the [`Arrow`] components,
/// and updates the arrow items accordingly.
#[derive(Default, Debug, Clone, Copy)]
pub struct BevyArrowsPlugin;

impl Plugin for BevyArrowsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, vec_arrow::on_attach_vec_arrow);
        app.add_systems(Update, vec_arrow::on_remove_vec_arrow);
    }
}

pub mod prelude {
    pub use crate::BevyArrowsPlugin;
}
