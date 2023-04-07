use perigee::prelude::*;

pub struct CheckpointEventRelayer {
    inner: Sender<(ColliderEvent, ColliderHandle)>,
    for_sensor: ColliderHandle,
}

impl CheckpointEventRelayer {
    pub fn new(
        event_sender: Sender<(ColliderEvent, ColliderHandle)>,
        for_sensor: ColliderHandle,
    ) -> Self {
        Self {
            inner: event_sender,
            for_sensor,
        }
    }
}

impl ColliderEventListener for CheckpointEventRelayer {
    fn on_collision_start(&mut self, other: &ColliderHandle) {
        let _send_result = self
            .inner
            .send((ColliderEvent::CollisionStart(*other), self.for_sensor));
    }

    fn on_collision_end(&mut self, other: &ColliderHandle) {
        let _send_result = self
            .inner
            .send((ColliderEvent::CollisionEnd(*other), self.for_sensor));
    }

    fn on_intersection_start(&mut self, other: &ColliderHandle) {
        let _send_result = self
            .inner
            .send((ColliderEvent::IntersectionStart(*other), self.for_sensor));
    }

    fn on_intersection_end(&mut self, other: &ColliderHandle) {
        let _send_result = self
            .inner
            .send((ColliderEvent::IntersectionEnd(*other), self.for_sensor));
    }

    fn on_contact_force_event(&mut self, other: &ColliderHandle, details: ContactForceEvent) {
        let _send_result = self.inner.send((
            ColliderEvent::ContactForceEvent(*other, details),
            self.for_sensor,
        ));
    }
}
