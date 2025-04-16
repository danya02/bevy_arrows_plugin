use bevy::prelude::*;

/// An arrow starting at the object's transform,
/// pointing at a particular position.
#[derive(Component)]
pub struct VecArrow {
    /// Where is the arrow pointing?
    pub target: Vec3,

    /// What coordinate system is the target in?
    pub target_coordinate_space: TargetCoordinateSpace,

    /// Thickness of the line in scene units.
    pub thickness: f32,

    /// Color of the line and the tip.
    pub color: Color,

    /// Thickness of the tip (diameter at the bottom of the arrow)
    pub tip_thickness: f32,

    /// Length of the tip
    pub tip_length: f32,
}

impl VecArrow {
    pub fn new(target: Vec3, target_coordinate_space: TargetCoordinateSpace) -> Self {
        Self {
            target,
            target_coordinate_space,
            thickness: 0.1,
            color: Color::WHITE,
            tip_thickness: 0.075,
            tip_length: 0.15,
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
pub(crate) struct VecArrowBody {}

/// This component is used by the plugin internally
/// and marks the tip of the arrow
/// (which is a cone).
#[derive(Component)]
pub(crate) struct VecArrowTip {}

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
    query: Query<(Entity, Option<&GlobalTransform>, &VecArrow), Added<VecArrow>>,
) {
    // When a vec-arrow is added,
    // we need to create a cylinder
    // and a cone.
    for (new_parent_entity, parent_global_transform, new_arrow) in query.iter() {
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
                    parent_global_transform.cloned(),
                    &new_arrow.target,
                    &new_arrow.target_coordinate_space,
                ),
                VecArrowBody {},
                Name::new(format!("VecArrowBody for {}", new_parent_entity)),
            ))
            .id();

        let tip = commands
            .spawn((
                Mesh3d(meshes.add(Cone::new(1.0, 1.0))),
                MeshMaterial3d(materials.add(new_arrow.color)),
                get_tip_transform(
                    parent_global_transform.cloned(),
                    &new_arrow.target,
                    &new_arrow.target_coordinate_space,
                    new_arrow.tip_length,
                    new_arrow.tip_thickness,
                ),
                Name::new(format!("VecArrowTip for {}", new_parent_entity)),
                VecArrowTip {},
            ))
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

pub(crate) fn update_vec_arrow(
    parent_transforms: Query<(&GlobalTransform, &VecArrow, &VecArrowParts)>,
    mut body_query: Query<
        (
            &mut Transform,
            &MeshMaterial3d<StandardMaterial>,
            &VecArrowBody,
        ),
        Without<VecArrowTip>,
    >,
    mut tip_query: Query<
        (
            &mut Transform,
            &MeshMaterial3d<StandardMaterial>,
            &VecArrowTip,
        ),
        Without<VecArrowBody>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (global_transform, vec_arrow, parts) in parent_transforms.iter() {
        let new_body_transform = get_body_transform(
            Some(*global_transform),
            &vec_arrow.target,
            &vec_arrow.target_coordinate_space,
        );
        let new_tip_transform = get_tip_transform(
            Some(*global_transform),
            &vec_arrow.target,
            &vec_arrow.target_coordinate_space,
            vec_arrow.tip_length,
            vec_arrow.tip_thickness,
        );

        let (mut body_transform, body_material, _) = body_query.get_mut(parts.body).unwrap();
        *body_transform = new_body_transform;
        if let Some(material) = materials.get_mut(&body_material.0) {
            material.base_color = vec_arrow.color;
        }

        let (mut tip_transform, tip_material, _) = tip_query.get_mut(parts.tip).unwrap();
        *tip_transform = new_tip_transform;
        if let Some(material) = materials.get_mut(&tip_material.0) {
            material.base_color = vec_arrow.color;
        }
    }
}

fn get_body_transform(
    parent_transform: Option<GlobalTransform>,
    target: &Vec3,
    target_coordinate_space: &TargetCoordinateSpace,
) -> Transform {
    // If the target vector is in the local coordinate system,
    // then it looks like a vector from the origin directly to the target.
    // However, if it's in the global coordinate system,
    // then the arrow is shifted in the opposite direction.
    let target = match target_coordinate_space {
        TargetCoordinateSpace::Local => *target,
        TargetCoordinateSpace::Global => {
            -parent_transform.unwrap_or_default().translation() + *target
        }
    };

    let Some(normalized) = target.try_normalize() else {
        // If the target is a zero vector,
        // return a transform with a zero scale.
        return Transform::from_scale(Vec3::ZERO);
    };

    // When pointing at a spot in the local transform,
    // the cylinder's position is just 1/2 of the target.
    let my_position = target / 2.0;

    let mut my_local_transform = Transform::from_translation(my_position);

    // After the look transform is applied, Apply a 90 degree rotation along X
    my_local_transform.rotate(Quat::from_rotation_arc(Vec3::Y, normalized));

    // The Y scale of the cylinder is equal to the distance
    // between the parent's position and the target
    // (because unscaled, the height is equal to 1)
    let my_scale = Vec3::new(1.0, target.length(), 1.0);
    let mut my_local_transform = my_local_transform.with_scale(my_scale);

    match target_coordinate_space {
        TargetCoordinateSpace::Global => {
            // If the target is in the global coordinate space,
            // then our local transform is already correct.
            // All we need to do is translate it to match the parent's origin.
            my_local_transform.translation += parent_transform.unwrap_or_default().translation();
            my_local_transform
        }
        TargetCoordinateSpace::Local => {
            // If the target is in the local coordinate space,
            // then we need to apply the parent's transform
            // to our current one.
            // We have to do this selectively, only doing translation and rotation.
            let parent_transform = parent_transform.unwrap_or_default();
            let mut my_global_transform = my_local_transform;
            my_global_transform.translation = parent_transform
                .rotation()
                .mul_vec3(my_global_transform.translation)
                + parent_transform.translation();
            my_global_transform.rotation = parent_transform
                .rotation()
                .mul_quat(my_global_transform.rotation);

            my_global_transform
        }
    }
}

fn get_tip_transform(
    parent_transform: Option<GlobalTransform>,
    target: &Vec3,
    target_coordinate_space: &TargetCoordinateSpace,
    tip_length: f32,
    tip_thickness: f32,
) -> Transform {
    // If the target vector is in the local coordinate system,
    // then it looks like a vector from the origin directly to the target.
    // However, if it's in the global coordinate system,
    // then the arrow is shifted in the opposite direction.
    let target = match target_coordinate_space {
        TargetCoordinateSpace::Local => *target,
        TargetCoordinateSpace::Global => {
            -parent_transform.unwrap_or_default().translation() + *target
        }
    };

    let Some(normalized) = target.try_normalize() else {
        // If the target is a zero vector,
        // return a transform with a zero scale.
        return Transform::from_scale(Vec3::ZERO);
    };

    let mut my_local_transform = Transform::from_translation(target);
    my_local_transform.rotate(Quat::from_rotation_arc(Vec3::Y, normalized));

    // X, Z transform to match the thickness,
    // Y transform to match the length
    my_local_transform.scale = Vec3::new(tip_thickness, tip_length, tip_thickness);

    match target_coordinate_space {
        TargetCoordinateSpace::Global => {
            // If the target is in the global coordinate space,
            // then our local transform is already correct,
            // so we return that.
            my_local_transform.translation += parent_transform.unwrap_or_default().translation();
            my_local_transform
        }
        TargetCoordinateSpace::Local => {
            // If the target is in the local coordinate space,
            // then we need to apply the parent's transform
            // to our current one.
            // We have to do this selectively, only doing translation and rotation.
            let parent_transform = parent_transform.unwrap_or_default();
            let mut my_global_transform = my_local_transform;

            my_global_transform.translation = parent_transform
                .rotation()
                .mul_vec3(my_global_transform.translation)
                + parent_transform.translation();
            my_global_transform.rotation = parent_transform
                .rotation()
                .mul_quat(my_global_transform.rotation);

            my_global_transform
        }
    }
}
