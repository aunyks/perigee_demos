use std::time::Duration;

use perigee::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::descriptor::Descriptor;

enum PlatformSensorEvent {
    OnIntersectionStart(ColliderHandle),
    OnIntersectionEnd(ColliderHandle),
}
struct PlatformSensorListener(Sender<PlatformSensorEvent>);
impl PhysicsEventListener for PlatformSensorListener {
    fn on_intersection_start(&mut self, other: &ColliderHandle) {
        let _send_result = self
            .0
            .send(PlatformSensorEvent::OnIntersectionStart(*other));
    }

    fn on_intersection_end(&mut self, other: &ColliderHandle) {
        let _send_result = self.0.send(PlatformSensorEvent::OnIntersectionEnd(*other));
    }
}

#[derive(Serialize, Deserialize)]
enum PlatformOperation {
    Transitioning,
    Waiting,
}

#[derive(Serialize, Deserialize)]
struct PlatformMovementState {
    pub operation: StateMachine<PlatformOperation>,
    pub clock: PassiveClock,
}

#[derive(Serialize, Deserialize)]
pub struct MovingPlatform<'a> {
    descriptor: Descriptor<'a>,
    sensor_name: &'a str,
    supported_bodies: Vec<RigidBodyHandle>,
    waypoints: Vec<Isometry<f32, UnitQuaternion<f32>, 3>>,
    waypoint_idx: usize,
    movement_state: PlatformMovementState,
    move_duration: Duration,
    wait_duration: Duration,
    sensor_local_iso: Isometry<f32, UnitQuaternion<f32>, 3>,
    #[serde(skip)]
    sensor_event_channel: EventChannel<PlatformSensorEvent>,
}

impl<'a> MovingPlatform<'a> {
    pub fn new(name: impl Into<Descriptor<'a>>, sensor_name: &'a str) -> Self {
        Self {
            sensor_name,
            descriptor: name.into(),
            supported_bodies: Vec::new(),
            waypoints: Vec::new(),
            waypoint_idx: 0,
            move_duration: Duration::from_secs_f32(3.0),
            wait_duration: Duration::from_secs_f32(2.0),
            sensor_local_iso: Isometry::identity(),
            movement_state: PlatformMovementState {
                operation: StateMachine::new(PlatformOperation::Transitioning),
                clock: PassiveClock::new(),
            },
            sensor_event_channel: EventChannel::with_capacity(0),
        }
    }

    pub fn initialize(
        &mut self,
        waypoints: Vec<Isometry<f32, UnitQuaternion<f32>, 3>>,
        physics: &mut PhysicsWorld,
    ) {
        self.waypoints = waypoints;

        if let Some(sensor_handle) = physics.named_sensors.handle_with_name(self.sensor_name) {
            physics.listen_to_collider(
                *sensor_handle,
                PlatformSensorListener(self.sensor_event_channel.clone_sender()),
            );
        }
        if let Some(sensor) = physics
            .named_sensors
            .handle_with_name(self.sensor_name)
            .and_then(|sensor_handle| physics.collider_set.get(*sensor_handle))
        {
            self.sensor_local_iso = self.waypoint(self.waypoint_idx).inverse() * sensor.position();
        }

        if let Some(plat_body) = physics
            .named_rigid_bodies
            .handle_with_name(self.descriptor.object_name())
            .and_then(|plat_handle| physics.rigid_body_set.get_mut(*plat_handle))
        {
            plat_body.set_position(self.waypoint(self.waypoint_idx), true);
        }
    }

    pub fn update(&mut self, physics: &mut PhysicsWorld, delta_seconds: f32) {
        self.movement_state.clock.tick(delta_seconds);

        match self.movement_state.operation.current_state() {
            &PlatformOperation::Transitioning => {
                let t = self.movement_state.clock.elapsed().as_secs_f32()
                    / self.move_duration.as_secs_f32();

                let current_position = self
                    .waypoint(self.waypoint_idx)
                    .lerp_slerp(&self.waypoint(self.next_waypoint_index()), t);

                let mut platform_dist_traveled = Vector3::zeros();

                if let Some(plat_handle) = physics
                    .named_rigid_bodies
                    .handle_with_name(self.descriptor.object_name())
                {
                    if let Some(plat_body) = physics.rigid_body_set.get_mut(*plat_handle) {
                        platform_dist_traveled =
                            current_position.translation.vector - plat_body.translation();
                        plat_body.set_position(current_position, true);
                    }
                }

                if let Some(sens_handle) = physics.named_sensors.handle_with_name(self.sensor_name)
                {
                    if let Some(sensor) = physics.collider_set.get_mut(*sens_handle) {
                        sensor.set_position(current_position * self.sensor_local_iso);
                    }
                }

                self.handle_sensor_events(&physics);

                for supported_body_handle in &mut self.supported_bodies {
                    if let Some(supported_body) =
                        physics.rigid_body_set.get_mut(*supported_body_handle)
                    {
                        supported_body.set_translation(
                            supported_body.translation() + platform_dist_traveled,
                            true,
                        );
                    }
                }

                if self.movement_state.clock.elapsed() >= self.move_duration {
                    self.movement_state
                        .operation
                        .transition_to(PlatformOperation::Waiting);
                    self.movement_state.clock.reset();
                }
            }
            &PlatformOperation::Waiting => {
                if self.movement_state.clock.elapsed() >= self.wait_duration {
                    self.movement_state
                        .operation
                        .transition_to(PlatformOperation::Transitioning);
                    self.movement_state.clock.reset();
                    self.waypoint_idx = self.next_waypoint_index();
                }
            }
        }
    }

    pub fn waypoint(&self, i: usize) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        self.waypoints[i]
    }

    pub fn next_waypoint_index(&self) -> usize {
        (self.waypoint_idx + 1) % self.waypoints.len()
    }

    pub fn add_supported_body(&mut self, new_body: RigidBodyHandle) {
        self.supported_bodies.push(new_body);
    }

    pub fn remove_supported_body(&mut self, body: RigidBodyHandle) {
        for i in 0..self.supported_bodies.len() {
            if self.supported_bodies[i] == body {
                self.supported_bodies.swap_remove(i);
            }
        }
    }

    pub fn handle_sensor_events(&mut self, physics: &PhysicsWorld) {
        while let Ok(sensor_event) = self.sensor_event_channel.get_message() {
            match sensor_event {
                PlatformSensorEvent::OnIntersectionStart(other_handle) => {
                    if let Some(rigid_body_handle) = physics
                        .collider_set
                        .get(other_handle)
                        .and_then(|collider| collider.parent())
                    {
                        if Some(&rigid_body_handle)
                            != physics
                                .named_rigid_bodies
                                .handle_with_name(self.descriptor.object_name())
                        {
                            self.add_supported_body(rigid_body_handle);
                        }
                    }
                }
                PlatformSensorEvent::OnIntersectionEnd(other_handle) => {
                    if let Some(rigid_body_handle) = physics
                        .collider_set
                        .get(other_handle)
                        .and_then(|collider| collider.parent())
                    {
                        if Some(&rigid_body_handle)
                            != physics
                                .named_rigid_bodies
                                .handle_with_name(self.descriptor.object_name())
                        {
                            self.remove_supported_body(rigid_body_handle);
                        }
                    }
                }
            }
        }
    }
}
