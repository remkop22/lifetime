pub trait IntoStatic {
    type Static: 'static;

    fn into_static(self) -> Self::Static;
}
