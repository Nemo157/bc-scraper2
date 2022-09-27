use std::time::Duration;
use crate::{
    phys::{Acceleration, Velocity},
    data::Data,
};

fn update_position(data: &mut Data, delta: Duration) {
    for entity in &mut data.entities {
        entity.position += entity.velocity * delta;
    }
}

fn update_velocity(data: &mut Data, delta: Duration) {
    for entity in &mut data.entities {
        if entity.is_under_mouse {
            entity.velocity = Velocity::default();
        } else {
            entity.velocity = (entity.velocity * 0.7 + entity.acceleration * delta).clamp(1000.0);
        }
    }
}

fn repel(data: &mut Data) {
    for id1 in data.entities.ids() {
        for id2 in data.entities.ids_after(id1) {
            let [entity1, entity2] = data.entities.index_many_mut([id1, id2]);
            let distance = entity1.position - entity2.position;
            let dsq = distance.euclid_squared().raw().max(0.001);
            let acceleration = Acceleration::from(distance.0 * 1000.0) / dsq;
            entity1.acceleration += acceleration;
            entity2.acceleration += -acceleration;
        }
    }
}

fn attract(data: &mut Data) {
    for rel in &data.relationships {
        let [entity1, entity2] = data.entities.index_many_mut({ let mut indexes = [rel.album, rel.user]; indexes.sort(); indexes });
        // TODO: Unit for attraction
        let attraction = Acceleration::from((entity2.position - entity1.position).0 * 2.0);
        entity1.acceleration += attraction;
        entity2.acceleration += -attraction;
    }
}

fn update_acc(data: &mut Data) {
    for entity in &mut data.entities {
        entity.acceleration = Acceleration::default();
    }
    repel(data);
    attract(data);
}

pub fn update(data: &mut Data, delta: Duration) {
    update_position(data, delta);
    update_acc(data);
    update_velocity(data, delta);
}
