/// This mod contains most common used arguments kinds for [Command] and [Query].
use super::Args;
use crate::status::ext::GrpcStatus;
use std::marker::PhantomData;

pub struct Get<D, T, E = GrpcStatus> {
    from: D,
    _data: (PhantomData<T>, PhantomData<E>),
}

impl<T, E, D> Get<D, T, E> {
    pub fn new(from: D) -> Self {
        Self {
            from,
            _data: (Default::default(), Default::default()),
        }
    }

    pub fn dst(&self) -> &D {
        &self.from
    }
}

impl<T, E, D> Args for Get<D, T, E> {
    type Output = Result<T, E>;
}

pub struct Set<D, E = GrpcStatus> {
    into: D,
    _data: PhantomData<E>,
}

impl<E, D> Set<D, E> {
    pub fn new(into: D) -> Self {
        Self {
            into,
            _data: Default::default(),
        }
    }

    pub fn dst(&self) -> &D {
        &self.into
    }
}

impl<E, D> Args for Set<D, E> {
    type Output = Result<(), E>;
}

pub struct Del<D, E = GrpcStatus> {
    dst: D,
    _data: PhantomData<E>,
}

impl<E, D> Del<D, E> {
    pub fn new(dst: D) -> Self {
        Self {
            dst,
            _data: Default::default(),
        }
    }

    pub fn dst(&self) -> &D {
        &self.dst
    }
}

impl<E, D> Args for Del<D, E> {
    type Output = Result<(), E>;
}

pub struct Put<D, E = GrpcStatus> {
    dst: D,
    _data: PhantomData<E>,
}

impl<E, D> Put<D, E> {
    pub fn new(dst: D) -> Self {
        Self {
            dst,
            _data: Default::default(),
        }
    }

    pub fn dst(&self) -> &D {
        &self.dst
    }
}

impl<E, D> Args for Put<D, E> {
    type Output = Result<(), E>;
}

// Redis operations arguments
pub type RedisGet<T, E = GrpcStatus> = Get<&'static redis::Client, T, E>;
pub type RedisSet<E = GrpcStatus> = Set<&'static redis::Client, E>;
pub type RedisDel<E = GrpcStatus> = Del<&'static redis::Client, E>;
pub type RedisPut<E = GrpcStatus> = Put<&'static redis::Client, E>;
