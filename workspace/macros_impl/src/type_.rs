use syn::{
    visit::{self, Visit},
    Lifetime, Type,
};

pub(crate) fn type_has_generic_lifetime(ty: &Type) -> bool {
    struct Visitor {
        has_generic_lifetime: bool,
    }
    impl Visit<'_> for Visitor {
        fn visit_lifetime(&mut self, lifetime: &Lifetime) {
            if lifetime.ident != "static" {
                self.has_generic_lifetime = true
            } else {
                visit::visit_lifetime(self, lifetime)
            }
        }
    }
    let mut visitor = Visitor {
        has_generic_lifetime: false,
    };
    Visit::visit_type(&mut visitor, ty);
    visitor.has_generic_lifetime
}
