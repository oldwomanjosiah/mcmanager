//! Interest and Event Flag Definitions

use inotify_sys as ffi;

#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EventFlag {
    #[doc(alias = "ACCESS")]
    Access = inotify_sys::IN_ACCESS,

    #[doc(alias = "MODIFY")]
    Write = ffi::IN_MODIFY,

    #[doc(alias = "ATTRIB")]
    Metadata = ffi::IN_ATTRIB,

    #[doc(alias = "CLOSE_WRITE")]
    CloseWrite = ffi::IN_CLOSE_WRITE,

    #[doc(alias = "CLOSE_NOWRITE")]
    CloseNoWrite = ffi::IN_CLOSE_NOWRITE,

    #[doc(alias = "OPEN")]
    Open = ffi::IN_OPEN,

    #[doc(alias = "MOVED_FROM")]
    MoveSource = ffi::IN_MOVED_FROM,

    #[doc(alias = "MOVED_TO")]
    MoveDestination = ffi::IN_MOVED_TO,

    #[doc(alias = "CREATE")]
    Create = ffi::IN_CREATE,

    #[doc(alias = "DELETE")]
    Delete = ffi::IN_DELETE,

    #[doc(alias = "DELETE_SELF")]
    DeleteSelf = ffi::IN_DELETE_SELF,

    #[doc(alias = "MODIFY_SELF")]
    MoveSelf = ffi::IN_MOVE_SELF,
}

#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InterestFlag {
    #[doc(alias = "ONLYDIR")]
    DirOnly = ffi::IN_ONLYDIR,

    #[doc(alias = "DONT_FOLLOW")]
    NoFollow = ffi::IN_DONT_FOLLOW,

    #[doc(alias = "EXCL_UNLINK")]
    NoUnlink = ffi::IN_EXCL_UNLINK,
    // TODO(josiah) this should not be allowed, since we manage watch states internaly
    // #[doc(alias = "MASK_ADD")]
    // UnifyFilter = ffi::IN_MASK_ADD,
    // TODO(josiah) this should be handled by the watch request api instead
    // Once = ffi::IN_ONESHOT,
    // TODO(josiah) this should not be allowed, since we manage watch states internaly
    // Replace = ffi::IN_MASK_CREATE,
}

/// Represents a masked selection of events which can be added to a file watch
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct EventMask(pub(crate) u32);

/// Selection of [`InterestFlag`]s which modify the types of events responded to by this watch
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Interests(pub(crate) u32);

/// A Full Filter for a watch request
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct EventFilter(pub(crate) u32);

impl From<EventMask> for EventFilter {
    fn from(mask: EventMask) -> Self {
        Self(mask.0)
    }
}

#[allow(non_upper_case_globals)]
impl EventMask {
    #[doc(alias = "MOVE")]
    pub const Move: EventMask = EventMask(ffi::IN_MOVE);

    #[doc(alias = "CLOSE")]
    pub const Close: EventMask = EventMask(ffi::IN_CLOSE);

    #[doc(alias = "ALL_EVENTS")]
    pub const Any: EventMask = EventMask(ffi::IN_ALL_EVENTS);

    pub fn filter(self, filter: InterestFlag) -> EventFilter {
        EventFilter(self.0 | filter as u32)
    }

    pub fn with_interests(self, filter: Interests) -> EventFilter {
        EventFilter(self.0 | filter.0)
    }
}

impl EventFilter {
    pub fn filter(self, filter: InterestFlag) -> EventFilter {
        Self(self.0 | filter as u32)
    }
}

/// Helper impls for converting bitflag types from their enum form to their final form
macro_rules! bitconvert {
    ($from:ident to $to:ident as $inner:ty) => {
        impl From<$from> for $to {
            fn from(it: $from) -> Self {
                Self(it as $inner)
            }
        }

        impl std::ops::BitOr for $from {
            type Output = $to;

            fn bitor(self, rhs: $from) -> $to {
                $to(self as $inner | rhs as $inner)
            }
        }

        impl<T: Into<$to>> std::ops::BitOr<T> for $to {
            type Output = $to;

            fn bitor(self, rhs: T) -> $to {
                $to(self.0 | rhs.into().0)
            }
        }

        impl std::ops::BitAnd for $from {
            type Output = $to;

            fn bitand(self, rhs: $from) -> $to {
                $to(self as $inner & rhs as $inner)
            }
        }

        impl<T: Into<$to>> std::ops::BitAnd<T> for $to {
            type Output = $to;

            fn bitand(self, rhs: T) -> $to {
                $to(self.0 & rhs.into().0)
            }
        }
    };
}

bitconvert!(EventFlag to EventMask as u32);
bitconvert!(InterestFlag to Interests as u32);
