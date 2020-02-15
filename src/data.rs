use hecs::{Entity, World};
use rand::{seq::SliceRandom, distributions::{Distribution, Uniform}};
use rand_distr::Poisson;

use crate::phys::{Position, Velocity, Acceleration};

#[derive(Debug)]
pub struct Relationship {
    pub from: Entity,
    pub to: Entity,
}

pub struct UnderMouse;

pub fn spawn_random(world: &mut World) {
    let mut rng = rand::thread_rng();
    let positions = Uniform::new(Position::new(200.0, 200.0), Position::new(400.0, 400.0));
    let velocities = Uniform::new(Velocity::new(-10.0, -10.0), Velocity::new(10.0, 10.0));

    let mut entities = Vec::new();
    for _ in 0..100 {
        entities.push(world.spawn((
            positions.sample(&mut rng),
            velocities.sample(&mut rng),
            Acceleration::default(),
        )));
    }

    for (i, from) in entities.iter().enumerate() {
        let count: u64 = Poisson::new((i / 20 + 1) as f64).unwrap().sample(&mut rng);
        let count = count.min(entities.len() as u64 / 2) as usize;
        for to in entities.choose_multiple(&mut rng, count) {
            if from == to { continue; }
            world.spawn((Relationship { from: *from, to: *to },));
        }
    }
}
