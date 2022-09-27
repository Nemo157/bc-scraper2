use hecs::World;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::time::Duration;
use crate::{
    phys::{Acceleration, Position, Velocity},
    data::{Relationship, UnderMouse},
};

fn update_pos(world: &mut World, delta: Duration) {
    for (_, (mut pos, vel)) in
        &mut world.query::<hecs::Without<UnderMouse, (&mut Position, &Velocity)>>()
    {
        pos += vel * delta;
    }
}

fn update_vel(world: &mut World, delta: Duration) {
    for (_, vel) in &mut world.query::<hecs::With<UnderMouse, &mut Velocity>>() {
        *vel = Velocity::default();
    }

    world
        .query::<hecs::Without<UnderMouse, (&mut Velocity, &Acceleration)>>()
        .iter_batched(1024)
        .collect::<Vec<_>>()
        .into_par_iter()
        .for_each(|batch| {
            batch.for_each(|(_, (mut vel, acc))| {
                vel *= 0.7;
                *vel = (*vel + acc * delta).clamp(1000.0);
            })
        });
}

fn repel(world: &mut World) {
    for (_, (pos1, mut acc1)) in &mut world.query::<(&Position, &mut Acceleration)>() {
        for (_, pos2) in &mut world.query::<&Position>() {
            let dist = pos1 - pos2;
            let dsq = dist.euclid_squared().raw().max(0.001);
            acc1 += Acceleration::from(dist.0 * 1000.0) / dsq;
        }
    }
}

fn attract(world: &mut World) {
    for (_, rel) in &mut world.query::<&Relationship>() {
        let (pos1, pos2) = (
            world.get::<Position>(rel.from).unwrap(),
            world.get::<Position>(rel.to).unwrap(),
        );
        // TODO: Unit for attraction
        let attraction = Acceleration::from((*pos2 - *pos1).0 * 2.0);
        let mut acc1 = world.get_mut::<Acceleration>(rel.from).unwrap();
        let mut acc2 = world.get_mut::<Acceleration>(rel.to).unwrap();
        *acc1 += attraction;
        *acc2 += -attraction;
    }
}

fn update_acc(world: &mut World) {
    for (_, acc) in &mut world.query::<&mut Acceleration>() {
        *acc = Acceleration::default();
    }
    repel(world);
    attract(world);
}

pub fn update(world: &mut World, delta: Duration) {
    update_pos(world, delta);
    update_acc(world);
    update_vel(world, delta);
}