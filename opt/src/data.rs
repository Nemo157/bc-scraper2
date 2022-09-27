use rand::{
    distributions::{Distribution, Uniform},
    seq::SliceRandom,
};
use rand_distr::Poisson;
use std::{collections::{BTreeSet, BTreeMap}, time::Instant};

use crate::phys::{Acceleration, Position, Velocity, Distance};

#[derive(Debug)]
pub enum Type {
    Album,
    User,
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct AlbumId(pub u64);

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct UserId(pub u64);

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct EntityId(u32);

#[derive(Debug)]
pub struct Drag {
    pub start_position: Position,
    pub start_time: Instant,
}

#[derive(Debug)]
pub struct Entity {
    pub position: Position,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub dragged: Option<Drag>,
    pub is_under_mouse: bool,
    pub data: EntityData,
}

#[derive(Debug)]
pub enum EntityData {
    Album(Album),
    User(User),
}

#[derive(Default, Debug)]
pub struct Camera {
    pub position: Position,
    pub zoom: f32,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Relationship {
    pub album: EntityId,
    pub user: EntityId,
}

#[derive(Default, Debug)]
pub struct Entities(Vec<Entity>);

#[derive(Default, Debug)]
pub struct Data {
    pub entities: Entities,
    pub relationships: BTreeSet<Relationship>,
    pub camera: Camera,
    pub albums: BTreeMap<AlbumId, EntityId>,
    pub users: BTreeMap<UserId, EntityId>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub id: AlbumId,
    pub url: String,
}

impl EntityData {
    fn at_random_location(self) -> Entity {
        let mut rng = rand::thread_rng();
        let positions = Uniform::new(Position::new(200.0, 200.0), Position::new(400.0, 400.0));
        let velocities = Uniform::new(Velocity::new(-10.0, -10.0), Velocity::new(10.0, 10.0));

        Entity {
            position: positions.sample(&mut rng),
            velocity: velocities.sample(&mut rng),
            acceleration: Acceleration::default(),
            dragged: None,
            is_under_mouse: false,
            data: self,
        }
    }

    fn at_random_location_near(self, position: Position) -> Entity {
        let mut rng = rand::thread_rng();
        let positions = Uniform::new(position - Distance::new(100.0, 100.0), position + Distance::new(100.0, 100.0));
        let velocities = Uniform::new(Velocity::new(-10.0, -10.0), Velocity::new(10.0, 10.0));

        Entity {
            position: positions.sample(&mut rng),
            velocity: velocities.sample(&mut rng),
            acceleration: Acceleration::default(),
            dragged: None,
            is_under_mouse: false,
            data: self,
        }
    }
}

impl Entities {
    pub fn add(&mut self, entity: Entity) -> EntityId {
        self.0.push(entity);
        EntityId(self.len() - 1)
    }

    fn len(&self) -> u32 {
        u32::try_from(self.0.len()).expect("too many entities")
    }

    pub fn ids(&self) -> impl Iterator<Item = EntityId> {
        (0..self.len()).map(EntityId)
    }

    pub fn ids_after(&self, start: EntityId) -> impl Iterator<Item = EntityId> {
        ((start.0 + 1)..self.len()).map(EntityId)
    }

    pub fn index_many_mut<const N: usize>(&mut self, indices: [EntityId; N]) -> [&mut Entity; N] {
        let indices = indices.map(|id| usize::try_from(id.0).unwrap());
        index_many::simple::index_many_mut(&mut self.0, indices)
    }
}

impl core::ops::Index<EntityId> for Entities {
    type Output = Entity;

    fn index(&self, id: EntityId) -> &Self::Output {
        &self.0[id.0 as usize]
    }
}

impl core::ops::IndexMut<EntityId> for Entities {
    fn index_mut(&mut self, id: EntityId) -> &mut Self::Output {
        &mut self.0[id.0 as usize]
    }
}

impl<'a> core::iter::IntoIterator for &'a mut Entities {
    type Item = &'a mut Entity;
    type IntoIter = core::slice::IterMut<'a, Entity>;
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<'a> core::iter::IntoIterator for &'a Entities {
    type Item = &'a Entity;
    type IntoIter = core::slice::Iter<'a, Entity>;
    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl Data {
    pub fn add_relationship(&mut self, album: &Album, user: &User) {
        let (album, user) = if let Some(&album) = self.albums.get(&album.id) {
            let &mut user = self.users
                .entry(user.id)
                .or_insert_with(|| {
                    self.entities.add(EntityData::User(user.clone()).at_random_location_near(self.entities[album].position))
                });
            (album, user)
        } else if let Some(&user) = self.users.get(&user.id) {
            let &mut album = self.albums
                .entry(album.id)
                .or_insert_with(|| {
                    self.entities.add(EntityData::Album(album.clone()).at_random_location_near(self.entities[user].position))
                });
            (album, user)
        } else {
            let &mut album = self.albums
                .entry(album.id)
                .or_insert_with(|| {
                    self.entities.add(EntityData::Album(album.clone()).at_random_location())
                });
            let &mut user = self.users
                .entry(user.id)
                .or_insert_with(|| {
                    self.entities.add(EntityData::User(user.clone()).at_random_location())
                });
            (album, user)
        };

        self.relationships.insert(Relationship { album, user });
    }

    pub fn spawn_random(&mut self, albums: u64, users: u64) {
        let mut rng = rand::thread_rng();

        let mut albums = Vec::from_iter((0..albums).map(|_| { let id = rand::random(); Album { id: AlbumId(id), url: format!("no://random/album/{id}") } }));
        let users = Vec::from_iter((0..users).map(|_| { let id = rand::random(); User { id: UserId(id), url: format!("no://random/user/{id}") } }));

        let mut linked_albums = Vec::new();

        for user in &users {
            let count: u64 = Poisson::new(20.0).unwrap().sample(&mut rng) as u64;
            for album in albums.drain(..(count as usize).min(albums.len())) {
                linked_albums.push(album.clone());
                self.add_relationship(&album, user);
            }
        }

        for user in &users {
            let count: u64 = Poisson::new(3.0).unwrap().sample(&mut rng) as u64;
            for album in linked_albums.choose_multiple(&mut rng, count as usize) {
                self.add_relationship(album, user);
            }
        }

        for album in &albums {
            let user = users.choose(&mut rng).unwrap();
            self.add_relationship(album, user);
        }
    }
}
