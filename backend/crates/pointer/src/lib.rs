struct Pointer<'a, T> {
    pub value: Option<&'a T>,
    pub path: &'a str,
}

impl<'a, T> Pointer<'a, T> {
    pub fn root(value: &'a T) -> Self {
        Pointer {
            value: Some(value),
            path: "/",
        }
    }

    pub fn descend(&self, key: &str) -> Self {}
}
