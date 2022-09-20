use hecs::{Entity, World};
use rand::{seq::SliceRandom, distributions::{Distribution, Uniform}};
use rand_distr::Poisson;

use crate::phys::{Position, Velocity, Acceleration};

#[derive(Debug)]
pub struct Relationship {
    pub from: Entity,
    pub to: Entity,
}

#[derive(Debug)]
pub struct UnderMouse;

#[derive(Debug)]
pub struct Dragged;

#[derive(Debug)]
pub struct Album;

#[derive(Debug)]
pub struct User;

#[derive(Debug)]
pub struct Camera;

pub fn spawn_random(world: &mut World) {
    let mut rng = rand::thread_rng();
    let positions = Uniform::new(Position::new(200.0, 200.0), Position::new(400.0, 400.0));
    let velocities = Uniform::new(Velocity::new(-10.0, -10.0), Velocity::new(10.0, 10.0));

    let mut albums = Vec::new();
    for _ in 0..100 {
        albums.push(world.spawn((
            Album,
            positions.sample(&mut rng),
            velocities.sample(&mut rng),
            Acceleration::default(),
        )));
    }

    let mut users = Vec::new();
    for _ in 0..5 {
        users.push(world.spawn((
            User,
            positions.sample(&mut rng),
            velocities.sample(&mut rng),
            Acceleration::default(),
        )));
    }

    let mut linked_albums = Vec::new();

    for from in &users {
        let count: u64 = Poisson::new(20.0).unwrap().sample(&mut rng);
        for to in albums.drain(..(count as usize).min(albums.len())) {
            linked_albums.push(to);
            world.spawn((Relationship { from: *from, to },));
        }
    }

    for from in &users {
        let count: u64 = Poisson::new(3.0).unwrap().sample(&mut rng);
        for to in linked_albums.choose_multiple(&mut rng, count as usize) {
            world.spawn((Relationship { from: *from, to: *to },));
        }
    }

    for from in &albums {
        let to = users.choose(&mut rng).unwrap();
        world.spawn((Relationship { from: *from, to: *to },));
    }
}
