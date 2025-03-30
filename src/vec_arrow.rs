use bevy::prelude::*;

/// An arrow starting at the object's transform,
/// pointing at a particular position.
#[derive(Component)]
pub struct VecArrow {
    /// Where is the arrow pointing?
    pub target: Vec3,

    /// What coordinate system is the target in?
    pub target_coordinate_space: TargetCoordinateSpace,

    pub thickness: f32,
    pub color: Color,

    pub tip_thickness: f32,
    pub tip_length: f32,
}

impl VecArrow {
    pub fn new(target: Vec3, target_coordinate_space: TargetCoordinateSpace) -> Self {
        Self {
            target,
            target_coordinate_space,
            thickness: 0.1,
            color: Color::WHITE,
            tip_thickness: 0.5,
            tip_length: 0.5,
        }
    }

    pub const fn with_thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

pub enum TargetCoordinateSpace {
    /// Global coordinate space
    Global,

    /// Local to the object
    Local,
}

/// This component is used by the plugin internally
/// and marks the main body of the arrow
/// (which is a cylinder).
#[derive(Component)]
struct VecArrowBody {}

/// This component is used by the plugin internally
/// to store the Entity ids for the arrow parts.
/// This is used when the arrow is removed
/// to find the other entities to also remove.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct VecArrowParts {
    body: Entity,
    tip: Entity,
}

pub(crate) fn on_attach_vec_arrow(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<
        (
            Entity,
            Option<&Transform>,
            Option<&GlobalTransform>,
            &VecArrow,
        ),
        Added<VecArrow>,
    >,
) {
    // When a vec-arrow is added,
    // we need to create a cylinder
    // and a cone.
    for (new_parent_entity, parent_transform, parent_global_transform, new_arrow) in query.iter() {
        // Ensure the parent has Visibility and Transform components
        commands
            .entity(new_parent_entity)
            .insert_if_new(Visibility::Inherited)
            .insert_if_new(Transform::default());

        let body = commands
            .spawn((
                Mesh3d(meshes.add(Cylinder::new(0.01, 1.0))),
                // Mesh3d(meshes.add(Cone::new(0.1, 1.0))),
                MeshMaterial3d(materials.add(new_arrow.color)),
                get_body_transform(
                    parent_transform,
                    &new_arrow.target,
                    &new_arrow.target_coordinate_space,
                ),
                VecArrowBody {},
                Name::new(format!("VecArrowBody for {}", new_parent_entity)),
            ))
            .set_parent(new_parent_entity)
            .id();

        let tip = commands
            .spawn((
                Mesh3d(meshes.add(Cone::new(0.1, 1.0))),
                MeshMaterial3d(materials.add(new_arrow.color)),
                get_tip_transform(
                    parent_transform,
                    &new_arrow.target,
                    &new_arrow.target_coordinate_space,
                ),
                Name::new(format!("VecArrowTip for {}", new_parent_entity)),
            ))
            .set_parent(new_parent_entity)
            .id();

        commands
            .entity(new_parent_entity)
            .insert(VecArrowParts { body, tip });
    }
}

pub(crate) fn on_remove_vec_arrow(
    mut commands: Commands,
    mut parents_with_removed_arrows: RemovedComponents<VecArrow>,
    parent_state_query: Query<Option<&VecArrowParts>>,
) {
    for entity in parents_with_removed_arrows.read() {
        // entity has just had its VecArrow component removed by the user.
        // If the entity has despawned completely, then we can't find its children.
        // Otherwise, we can read the VecArrowParts,
        // which we use to keep track of the arrow components.
        if let Ok(Some(VecArrowParts { body, tip })) = parent_state_query.get(entity) {
            // Despawn the arrow parts
            commands.entity(*body).despawn();
            commands.entity(*tip).despawn();

            // Remove VecArrowParts from the parent
            commands.entity(entity).remove::<VecArrowParts>();
        }
    }
}

fn get_body_transform(
    parent_transform: Option<&Transform>,
    target: &Vec3,
    target_coordinate_space: &TargetCoordinateSpace,
) -> Transform {
    match target_coordinate_space {
        TargetCoordinateSpace::Global => todo!(),
        TargetCoordinateSpace::Local => {
            let Some(normalized) = target.try_normalize() else {
                // If the target is a zero vector,
                // return a transform with a zero scale.
                return Transform::from_scale(Vec3::ZERO);
            };

            // When pointing at a spot in the local transform,
            // the cylinder's position is just 1/2 of the target.
            let my_position = target / 2.0;

            let mut my_transform = Transform::from_translation(my_position);

            // After the look transform is applied, Apply a 90 degree rotation along X
            my_transform.rotate(Quat::from_rotation_arc(Vec3::Y, normalized));

            // The Y scale of the cylinder is equal to the distance
            // between the parent's position and the target
            // (because unscaled, the height is equal to 1)
            let my_scale = Vec3::new(1.0, target.length(), 1.0);
            let my_transform = my_transform.with_scale(my_scale);

            my_transform
        }
    }
}

fn get_tip_transform(
    parent_transform: Option<&Transform>,
    target: &Vec3,
    target_coordinate_space: &TargetCoordinateSpace,
) -> Transform {
    match target_coordinate_space {
        TargetCoordinateSpace::Global => todo!(),
        TargetCoordinateSpace::Local => {
            let Some(normalized) = target.try_normalize() else {
                // If the target is a zero vector,
                // return a transform with a zero scale.
                return Transform::from_scale(Vec3::ZERO);
            };

            let mut my_transform = Transform::from_translation(*target);
            my_transform.rotate(Quat::from_rotation_arc(Vec3::Y, normalized));
            my_transform
        }
    }
}
