use rand::{
    distributions::uniform::{SampleBorrow, SampleUniform, UniformFloat, UniformSampler},
    Rng,
};

use super::{Acceleration, Position, Vec2, Velocity};

#[derive(Clone, Copy, Debug)]
pub struct UniformVec2(UniformFloat<f32>, UniformFloat<f32>);

#[derive(Clone, Copy, Debug)]
pub struct UniformPosition(UniformVec2);

#[derive(Clone, Copy, Debug)]
pub struct UniformVelocity(UniformVec2);

#[derive(Clone, Copy, Debug)]
pub struct UniformAcceleration(UniformVec2);

impl UniformSampler for UniformVec2 {
    type X = Vec2;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformVec2(
            UniformFloat::<f32>::new(low.borrow().x.raw(), high.borrow().x.raw()),
            UniformFloat::<f32>::new(low.borrow().y.raw(), high.borrow().y.raw()),
        )
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformSampler::new(low, high)
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Vec2::from((self.0.sample(rng), self.1.sample(rng)))
    }
}

impl UniformSampler for UniformPosition {
    type X = Position;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformPosition(UniformVec2::new(low.borrow().0, high.borrow().0))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformSampler::new(low, high)
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Position::from(self.0.sample(rng))
    }
}

impl UniformSampler for UniformVelocity {
    type X = Velocity;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformVelocity(UniformVec2::new(low.borrow().0, high.borrow().0))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformSampler::new(low, high)
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Velocity::from(self.0.sample(rng))
    }
}

impl UniformSampler for UniformAcceleration {
    type X = Acceleration;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformAcceleration(UniformVec2::new(low.borrow().0, high.borrow().0))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformSampler::new(low, high)
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Acceleration::from(self.0.sample(rng))
    }
}

impl SampleUniform for Vec2 {
    type Sampler = UniformVec2;
}

impl SampleUniform for Position {
    type Sampler = UniformPosition;
}

impl SampleUniform for Velocity {
    type Sampler = UniformVelocity;
}

impl SampleUniform for Acceleration {
    type Sampler = UniformAcceleration;
}
