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
    DirOnly = ffi::IN_ONLYDIR,
    NoFollow = ffi::IN_DONT_FOLLOW,
    NoUnlink = ffi::IN_EXCL_UNLINK,
    UnifyFilter = ffi::IN_MASK_ADD,
    // TODO(josiah) this should be handled by the watch request api instead?
    // Once = ffi::IN_ONESHOT,
}

/// Represents a masked selection of events which can be added to a file watch
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct EventMask(u32);

/// Selection of [`InterestFlag`]s which modify the types of events responded to by this watch
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Interests(u32);

/// A Full Filter for a watch request
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct EventFilter(u32);

impl From<EventMask> for EventFilter {
    fn from(mask: EventMask) -> Self {
        Self(mask.0)
    }
}

#[allow(non_upper_case_globals)]
impl EventMask {
    #[doc(alias = "MOVE")]
    const Move: EventMask = EventMask(ffi::IN_MOVE);

    #[doc(alias = "CLOSE")]
    const Close: EventMask = EventMask(ffi::IN_CLOSE);

    #[doc(alias = "ALL_EVENTS")]
    const Any: EventMask = EventMask(ffi::IN_ALL_EVENTS);

    pub fn filter(self, filter: InterestFlag) -> EventFilter {
        EventFilter(self.0 | filter as u32)
    }

    pub fn with_intersts(self, filter: Interests) -> EventFilter {
        EventFilter(self.0 | filter.0)
    }
}

impl EventFilter {
    pub fn filter(self, filter: InterestFlag) -> EventFilter {
        Self(self.0 | filter as u32)
    }
}

#[allow(non_upper_case_globals)]
impl EventFlag {
    #[doc(alias = "MOVE")]
    const Move: EventMask = EventMask::Move;

    #[doc(alias = "CLOSE")]
    const Close: EventMask = EventMask::Close;

    #[doc(alias = "ALL_EVENTS")]
    const Any: EventMask = EventMask::Any;
}

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
