use hecs::{Entity, World, EntityBuilder, DynamicBundle};
use rand::{
    distributions::{Distribution, Uniform},
    seq::SliceRandom,
};
use rand_distr::Poisson;
use std::collections::{BTreeSet, BTreeMap};

use crate::phys::{Acceleration, Position, Velocity};

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

trait WorldExt {
    fn spawn_at_random_location(&mut self, components: impl DynamicBundle) -> Entity;
}

impl WorldExt for World {
    fn spawn_at_random_location(&mut self, components: impl DynamicBundle) -> Entity {
        let mut rng = rand::thread_rng();
        let positions = Uniform::new(Position::new(200.0, 200.0), Position::new(400.0, 400.0));
        let velocities = Uniform::new(Velocity::new(-10.0, -10.0), Velocity::new(10.0, 10.0));

        self.spawn(EntityBuilder::new().add_bundle(components).add_bundle((
            positions.sample(&mut rng),
            velocities.sample(&mut rng),
            Acceleration::default(),
        )).build())
    }
}

pub struct Loader {
    albums: BTreeMap<u64, Entity>,
    users: BTreeMap<u64, Entity>,
    relationships: BTreeSet<(u64, u64)>,
}

impl Loader {
    pub fn new() -> Self {
        Self { albums: BTreeMap::new(), users: BTreeMap::new(), relationships: BTreeSet::new() }
    }

    pub fn add_relationship(&mut self, world: &mut World, album_id: u64, user_id: u64) {
        if self.relationships.insert((user_id, album_id)) {
            let &mut album = self.albums.entry(album_id).or_insert_with(|| world.spawn_at_random_location((Album,)));
            let &mut user = self.users.entry(user_id).or_insert_with(|| world.spawn_at_random_location((User,)));
            world.spawn((Relationship { from: user, to: album },));
        }
    }

    pub fn spawn_random(&mut self, world: &mut World) {
        let mut rng = rand::thread_rng();

        let mut albums = Vec::from_iter(rand::random::<[u64; 100]>());
        let users = Vec::from_iter(rand::random::<[u64; 5]>());

        let mut linked_albums = Vec::new();

        for &from in &users {
            let count: u64 = Poisson::new(20.0).unwrap().sample(&mut rng) as u64;
            for to in albums.drain(..(count as usize).min(albums.len())) {
                linked_albums.push(to);
                self.add_relationship(world, from, to);
            }
        }

        for &from in &users {
            let count: u64 = Poisson::new(3.0).unwrap().sample(&mut rng) as u64;
            for &to in linked_albums.choose_multiple(&mut rng, count as usize) {
                self.add_relationship(world, from, to);
            }
        }

        for &from in &albums {
            let &to = users.choose(&mut rng).unwrap();
            self.add_relationship(world, to, from);
        }
    }
}
