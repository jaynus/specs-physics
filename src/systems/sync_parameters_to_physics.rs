use nalgebra::RealField;
use specs::{Read, Resources, System, SystemData, WriteExpect};

use crate::{parameters::*, Physics};
use std::marker::PhantomData;

/// The `SyncParametersToPhysicsSystem` synchronises the simulation parameters
/// with the nphysics `World`.
pub struct SyncParametersToPhysicsSystem<N> {
    n_marker: PhantomData<N>,
}

impl<'s, N: RealField> System<'s> for SyncParametersToPhysicsSystem<N> {
    type SystemData = (
        Option<Read<'s, Gravity<N>>>,
        Option<Read<'s, PhysicsProfilingEnabled>>,
        Option<Read<'s, PhysicsIntegrationParameters<N>>>,
        WriteExpect<'s, Physics<N>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (gravity, profiling, integration_params, mut physics) = data;

        // if a Gravity resource exists, synchronise its values with the nphysics World
        if let Some(gravity) = gravity {
            if gravity.0 != *physics.gravity() {
                info!(
                    "Global physics gravity modified from {}, updating to {}.",
                    physics.gravity(),
                    gravity.0
                );
                physics.world.set_gravity(gravity.0);
            }
        }

        if let Some(enable_profiling) = profiling {
            if enable_profiling.0 != physics.performance_counters().enabled() {
                if enable_profiling.0 {
                    info!("Physics performance counters enabled.");
                    physics.world.enable_performance_counters();
                } else {
                    info!("Physics performance counters disabled.");
                    physics.world.disable_performance_counters();
                }
            }
        }

        if let Some(params) = integration_params {
            if *params != *physics.integration_parameters() {
                params.apply(physics.world.integration_parameters_mut());
                info!("Integration parameters have been updated.");
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        info!("SyncParametersToPhysicsSystem.setup");
        Self::SystemData::setup(res);

        // initialise required resources
        res.entry::<Physics<N>>().or_insert_with(Physics::default);
    }
}

impl<N> Default for SyncParametersToPhysicsSystem<N>
where
    N: RealField,
{
    fn default() -> Self {
        Self {
            n_marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Physics;
    use nalgebra::Vector3;
    use specs::{DispatcherBuilder, World};

    #[test]
    fn update_gravity() {
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(
                SyncParametersToPhysicsSystem::<f32>::default(),
                "sync_parameters_to_physics_system",
                &[],
            )
            .build();
        dispatcher.setup(&mut world.res);

        world.add_resource(Gravity(Vector3::<f32>::new(1.0, 2.0, 3.0).into()));
        dispatcher.dispatch(&mut world.res);

        let physics = world.read_resource::<Physics<f32>>();
        assert_eq!(physics.world.gravity().x, 1.0);
        assert_eq!(physics.world.gravity().y, 2.0);
        assert_eq!(physics.world.gravity().z, 3.0);
    }

}
