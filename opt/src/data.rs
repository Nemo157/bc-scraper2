use rand::{
    distributions::{Distribution, Uniform},
    seq::SliceRandom,
};
use rand_distr::Poisson;
use std::{sync::Arc, time::Instant};

use crate::phys::{Acceleration, Position, Velocity, Distance};

#[derive(Debug)]
pub enum Type {
    Album,
    User,
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct AlbumId(pub u64);

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct UserId(pub u64);

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct EntityId(u32);

#[derive(Clone, Debug)]
pub struct Drag {
    pub start_position: Position,
    pub start_time: Instant,
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub position: Position,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub dragged: Option<Drag>,
    pub is_under_mouse: bool,
    pub is_scraped: bool,
    pub data: Arc<EntityData>,
    pub related: im::HashSet<EntityId>,
}

#[derive(Debug)]
pub enum EntityData {
    Album(Album),
    User(User),
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Relationship {
    pub album: EntityId,
    pub user: EntityId,
}

#[derive(Default, Debug)]
pub struct Entities(Vec<Entity>);

impl Clone for Entities {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}

#[derive(Default, Debug)]
pub struct Data {
    pub entities: Entities,
    pub relationships: im::HashSet<Relationship>,
    pub albums: im::HashMap<AlbumId, EntityId>,
    pub users: im::HashMap<UserId, EntityId>,
}

impl Clone for Data {
    fn clone(&self) -> Self {
        Self {
            entities: self.entities.clone(),
            relationships: self.relationships.clone(),
            albums: self.albums.clone(),
            users: self.users.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.entities.clone_from(&source.entities);
        self.relationships.clone_from(&source.relationships);
        self.albums.clone_from(&source.albums);
        self.users.clone_from(&source.users);
    }
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
        let positions = Uniform::new(Position::new(0.0, 0.0), Position::new(800.0, 600.0));
        let velocities = Uniform::new(Velocity::new(-10.0, -10.0), Velocity::new(10.0, 10.0));

        Entity {
            position: positions.sample(&mut rng),
            velocity: velocities.sample(&mut rng),
            acceleration: Acceleration::default(),
            dragged: None,
            is_under_mouse: false,
            is_scraped: false,
            data: Arc::new(self),
            related: im::HashSet::new(),
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
            is_scraped: false,
            data: Arc::new(self),
            related: im::HashSet::new(),
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

    pub fn index_pair(&mut self, i: EntityId, j: EntityId) -> (&mut Entity, &mut Entity) {
        let (i, j) = (i.0 as usize, j.0 as usize);
        if i < j {
            let (left, right) = self.0.split_at_mut(i + 1);
            (&mut left[i], &mut right[j - i - 1])
        } else {
            let (left, right) = self.0.split_at_mut(j + 1);
            (&mut right[i - j - 1], &mut left[j])
        }
    }

    pub fn combinations(&mut self, f: impl Fn(&mut Entity, &mut Entity)) {
        for i in 0..self.0.len() {
            if let ([.., entity1], right) = self.0.split_at_mut(i + 1) {
                for entity2 in right {
                    f(entity1, entity2);
                }
            }
        }
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
        self.0.iter_mut()
    }
}

impl<'a> rayon::iter::IntoParallelIterator for &'a mut Entities {
    type Item = &'a mut Entity;
    type Iter = rayon::slice::IterMut<'a, Entity>;
    fn into_par_iter(self) -> Self::Iter {
        (&mut self.0).into_par_iter()
    }
}

impl<'a> core::iter::IntoIterator for &'a Entities {
    type Item = &'a Entity;
    type IntoIter = core::slice::Iter<'a, Entity>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
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
        self.entities[album].related.insert(user);
        self.entities[user].related.insert(album);
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
