use crate::layer::*;

// Combined auth is a type alias of cookie auth,
// since cookie auth handler receives Request<B>,
// which is required by bearer auth and basic auth etc.

pub type CombinedAuthLayer<Auth> = CookieAuthLayer<Auth>;

pub trait AuthCombined<B>: AuthCookie<B> {}

impl<B, F> AuthCombined<B> for F where F: AuthCookie<B> {}

pub type AsyncCombinedAuthLayer<Auth> = AsyncCookieAuthLayer<Auth>;

pub trait AsyncAuthCombined<B>: AsyncAuthCookie<B> {}

impl<B, F> AsyncAuthCombined<B> for F where F: AsyncAuthCookie<B> {}
