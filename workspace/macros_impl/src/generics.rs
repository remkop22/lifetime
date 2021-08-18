use syn::{GenericParam, Generics, Lifetime, LifetimeDef};

pub(crate) fn assert_generics_are_supported(generics: &Generics) {
    for param in &generics.params {
        match param {
            GenericParam::Type(type_param) => {
                panic!(
                    "Generic type parameters are not supported. The type parameter is {}",
                    type_param.ident
                )
            }
            GenericParam::Lifetime(lifetime_param) => {
                if !lifetime_param.bounds.is_empty() {
                    panic!("Generic lifetime parameters with bounds are not supported. The lifetime with bounds is '{}", lifetime_param.lifetime.ident)
                }
            }
            GenericParam::Const(const_param) => {
                panic!(
                    "Const generic parameters are not supported. The const parameter is {}",
                    const_param.ident
                )
            }
        }
    }
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
