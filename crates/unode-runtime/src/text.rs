use std::fmt;
use std::sync::Arc;

#[derive(Clone)]
pub enum DeferredText {
    Static(String),
    Dynamic(Arc<dyn Fn() -> String + Send + Sync>),
}

impl DeferredText {
    pub fn dynamic<F>(resolver: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        Self::Dynamic(Arc::new(resolver))
    }

    pub fn resolve(&self) -> String {
        match self {
            Self::Static(value) => value.clone(),
            Self::Dynamic(resolver) => resolver(),
        }
    }
}

impl fmt::Debug for DeferredText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(value) => f.debug_tuple("DeferredText::Static").field(value).finish(),
            Self::Dynamic(_) => f.write_str("DeferredText::Dynamic(<resolver>)"),
        }
    }
}

impl From<String> for DeferredText {
    fn from(value: String) -> Self {
        Self::Static(value)
    }
}

impl From<&str> for DeferredText {
    fn from(value: &str) -> Self {
        Self::Static(value.to_string())
    }
}
