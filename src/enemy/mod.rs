use bevy::{prelude::{Component, Vec2, Handle, Image, AssetServer, Assets, IVec2, Transform, Query}, sprite::TextureAtlas};
use bevy_ecs_ldtk::{prelude::{LdtkEntity, LayerInstance, TilesetDefinition, FieldValue}, EntityInstance, utils::ldtk_pixel_coords_to_translation_pivoted};

use crate::player::Direction;

pub mod knight;

pub struct PatrolPointReached(Patrol);

#[derive(Clone, PartialEq, Debug, Default, Component)]
pub struct Patrol {
    pub points: Vec<Vec2>,
    pub index: usize,
    pub forward: bool,
}

impl LdtkEntity for Patrol {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlas>,
    ) -> Patrol {
        let mut points = Vec::new();
        points.push(ldtk_pixel_coords_to_translation_pivoted(
            entity_instance.px,
            layer_instance.c_hei * layer_instance.grid_size,
            IVec2::new(entity_instance.width, entity_instance.height),
            entity_instance.pivot,
        ));

        let ldtk_patrol = entity_instance
            .field_instances
            .iter()
            .find(|f| f.identifier == *"Patrol")
            .unwrap();
        if let FieldValue::Points(ldtk_points) = &ldtk_patrol.value {
            for ldtk_point in ldtk_points {
                if let Some(ldtk_point) = ldtk_point {
                    // The +1 is necessary here due to the pivot of the entities in the sample
                    // file.
                    // The patrols set up in the file look flat and grounded,
                    // but technically they're not if you consider the pivot,
                    // which is at the bottom-center for the skulls.
                    let pixel_coords = (ldtk_point.as_vec2() + Vec2::new(0.5, 1.))
                        * Vec2::splat(layer_instance.grid_size as f32);

                    points.push(ldtk_pixel_coords_to_translation_pivoted(
                        pixel_coords.as_ivec2(),
                        layer_instance.c_hei * layer_instance.grid_size,
                        IVec2::new(entity_instance.width, entity_instance.height),
                        entity_instance.pivot,
                    ));
                }
            }
        }

        Patrol {
            points,
            index: 1,
            forward: true,
        }
    }
}

fn patrol(
    mut query: Query<(&mut Patrol, &mut Direction)>
    mut patrol_point_reach: EventWriter<
) {
    for (mut patrol, mut direction) in query.iter_mut() {
        if patrol.points.len() > 1 {
            if patrol.index == 0 {
                patrol.forward = true;
            } else if patrol.index == patrol.points.len() - 1 {
                patrol.forward = false;
            }


        }



    }
}