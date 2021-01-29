use futures::{TryFuture, TryFutureExt};
use std::time::SystemTime;

/// Keeps a single value
pub enum TtlCache<T> {
    Empty,
    Filled { value: T },
}

pub trait HasTtl {
    fn valid_until(&self) -> SystemTime;
}

impl<T> Default for TtlCache<T> {
    fn default() -> Self {
        TtlCache::Empty
    }
}

impl<T> TtlCache<T> {
    fn insert(&mut self, value: T) -> &mut T {
        *self = TtlCache::Filled { value };
        match self {
            TtlCache::Filled { value } => value,
            _ => unreachable!(),
        }
    }
}

impl<T: HasTtl> TtlCache<T> {
    fn clear_if_expired(&mut self, now: SystemTime) {
        if let TtlCache::Filled { value } = self {
            if now > value.valid_until() {
                *self = TtlCache::Empty
            }
        }
    }

    /// Immediately returns (a reference to) the current value if it is still valid,
    /// otherwise calls `update` to refresh it. Only [`Ok`] results are cached.
    pub async fn try_get_or_update<Fut: TryFuture<Ok = T>>(
        &mut self,
        valid_until_at_least: SystemTime,
        update: impl FnOnce() -> Fut,
    ) -> Result<&mut T, Fut::Error> {
        self.clear_if_expired(valid_until_at_least);
        Ok(match self {
            TtlCache::Empty => self.insert(update().into_future().await?),
            TtlCache::Filled { value } => value,
        })
    }
}
