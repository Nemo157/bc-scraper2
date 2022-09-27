use hecs::{Entity, World, EntityBuilder, DynamicBundle};
use rand::{
    distributions::{Distribution, Uniform},
    seq::SliceRandom,
};
use rand_distr::Poisson;
use std::{collections::{BTreeSet, BTreeMap}, time::Instant};

use crate::phys::{Acceleration, Position, Velocity};

#[derive(Debug)]
pub struct Relationship {
    pub from: Entity,
    pub to: Entity,
}

#[derive(Debug)]
pub struct UnderMouse;

#[derive(Debug)]
pub struct Dragged(pub Position, pub Instant);

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub id: u64,
    pub url: String,
}

#[derive(Debug)]
pub struct Camera;

#[derive(Debug)]
pub struct Zoom(pub f32);

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

    pub fn add_relationship(&mut self, world: &mut World, album: &Album, user: &User) {
        if self.relationships.insert((user.id, album.id)) {
            let &mut album = self.albums.entry(album.id).or_insert_with(|| world.spawn_at_random_location((album.clone(),)));
            let &mut user = self.users.entry(user.id).or_insert_with(|| world.spawn_at_random_location((user.clone(),)));
            world.spawn((Relationship { from: user, to: album },));
        }
    }

    pub fn spawn_random(&mut self, world: &mut World, albums: u64, users: u64) {
        let mut rng = rand::thread_rng();

        let mut albums = Vec::from_iter((0..albums).map(|_| { let id = rand::random(); Album { id, url: format!("no://random/album/{id}") } }));
        let users = Vec::from_iter((0..users).map(|_| { let id = rand::random(); User { id, url: format!("no://random/user/{id}") } }));

        let mut linked_albums = Vec::new();

        for user in &users {
            let count: u64 = Poisson::new(20.0).unwrap().sample(&mut rng) as u64;
            for album in albums.drain(..(count as usize).min(albums.len())) {
                linked_albums.push(album.clone());
                self.add_relationship(world, &album, user);
            }
        }

        for user in &users {
            let count: u64 = Poisson::new(3.0).unwrap().sample(&mut rng) as u64;
            for album in linked_albums.choose_multiple(&mut rng, count as usize) {
                self.add_relationship(world, album, user);
            }
        }

        for album in &albums {
            let user = users.choose(&mut rng).unwrap();
            self.add_relationship(world, album, user);
        }
    }
}
