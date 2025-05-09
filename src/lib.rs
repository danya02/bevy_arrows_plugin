pub mod vec_arrow;
use bevy::app::{Plugin, PostUpdate};

/// This plugin adds systems that keep track of the [`vec_arrow::VecArrow`] components,
/// and updates the arrow items accordingly.
#[derive(Default, Debug, Clone, Copy)]
pub struct BevyArrowsPlugin;

impl Plugin for BevyArrowsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(PostUpdate, vec_arrow::on_attach_vec_arrow);
        app.add_systems(PostUpdate, vec_arrow::on_remove_vec_arrow);
        app.add_systems(PostUpdate, vec_arrow::update_vec_arrow);
    }
}

pub mod prelude {
    pub use crate::BevyArrowsPlugin;
    pub use crate::vec_arrow::VecArrow;
}
