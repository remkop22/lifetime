use syn::{GenericParam, Generics, Lifetime, LifetimeDef};

pub(crate) fn has_generic_type(generics: &Generics) -> bool {
    generics
        .params
        .iter()
        .any(|p| matches!(p, GenericParam::Type(_)))
}

pub(crate) fn add_lifetime(mut generics: Generics, lifetime: Lifetime) -> Generics {
    generics
        .params
        .insert(0, GenericParam::Lifetime(LifetimeDef::new(lifetime)));
    generics
}

pub(crate) fn replace_lifetimes(mut generics: Generics, new: &Lifetime) -> Generics {
    for old in generics.lifetimes_mut() {
        *old = LifetimeDef::new(new.clone());
    }
    generics
}
