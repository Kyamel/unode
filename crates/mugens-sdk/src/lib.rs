pub mod permissions;

pub use permissions::{mugens_permission, MugensPermission};

pub mod prelude {
    pub use crate::permissions::{mugens, mugens_permission, MugensPermission};
    pub use unode_sdk::prelude::*;
}
