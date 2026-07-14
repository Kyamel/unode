pub mod permissions;

pub use permissions::{MugensPermission, mugens_permission};

pub mod prelude {
    pub use crate::permissions::{MugensPermission, mugens, mugens_permission};
    pub use unode_sdk::prelude::*;
}
