use rayon::iter::{IntoParallelIterator, ParallelIterator};
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
    (&mut data.entities).into_par_iter().for_each(|entity| {
        if entity.is_under_mouse {
            entity.velocity = Velocity::default();
        } else {
            entity.velocity = (entity.velocity * 0.7 + entity.acceleration * delta).clamp(1000.0);
        }
    });
}

fn repel(data: &mut Data) {
    data.entities
        .combinations(|entity1, entity2| {
            let distance = entity1.position - entity2.position;
            let dsq = distance.euclid_squared().raw().max(0.001);
            let acceleration = Acceleration::from(distance.0 * 1000.0) / dsq;
            entity1.acceleration += acceleration;
            entity2.acceleration += -acceleration;
        });
}

fn attract(data: &mut Data) {
    for rel in &data.relationships {
        let (album, user) = data.entities.index_pair(rel.album, rel.user);
        // TODO: Unit for attraction
        let attraction = Acceleration::from((user.position - album.position).0 * 2.0);
        album.acceleration += attraction;
        user.acceleration += -attraction;
    }
}

fn update_acc(data: &mut Data) {
    (&mut data.entities).into_par_iter().for_each(|entity| {
        entity.acceleration = Acceleration::default();
    });
    repel(data);
    attract(data);
}

pub fn update(data: &mut Data, delta: Duration) {
    update_position(data, delta);
    update_acc(data);
    update_velocity(data, delta);
}
