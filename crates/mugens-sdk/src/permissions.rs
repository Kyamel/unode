pub use unode_sdk::{PermissionRequestBuilder, permission as custom_permission};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MugensPermission {
    CatalogRead,
    LibraryRead,
    LibraryWrite,
    ReaderRead,
    SessionRead,
    SessionWrite,
}

impl MugensPermission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CatalogRead => "catalog.read",
            Self::LibraryRead => "library.read",
            Self::LibraryWrite => "library.write",
            Self::ReaderRead => "reader.read",
            Self::SessionRead => "session.read",
            Self::SessionWrite => "session.write",
        }
    }
}

pub fn mugens_permission(permission: MugensPermission) -> PermissionRequestBuilder {
    custom_permission(permission.as_str())
}

pub mod mugens {
    use super::{MugensPermission, PermissionRequestBuilder, mugens_permission};

    pub fn catalog_read() -> PermissionRequestBuilder {
        mugens_permission(MugensPermission::CatalogRead)
    }

    pub fn library_read() -> PermissionRequestBuilder {
        mugens_permission(MugensPermission::LibraryRead)
    }

    pub fn library_write() -> PermissionRequestBuilder {
        mugens_permission(MugensPermission::LibraryWrite)
    }

    pub fn reader_read() -> PermissionRequestBuilder {
        mugens_permission(MugensPermission::ReaderRead)
    }

    pub fn session_read() -> PermissionRequestBuilder {
        mugens_permission(MugensPermission::SessionRead)
    }

    pub fn session_write() -> PermissionRequestBuilder {
        mugens_permission(MugensPermission::SessionWrite)
    }
}
