use crate::ffi::{AVChromaLocation::*, *};

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde_derive::Serialize, serde_derive::Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "serde_", rename_all = "kebab-case"))]
pub enum Location {
	Unspecified,
	Left,
	Center,
	TopLeft,
	Top,
	BottomLeft,
	Bottom,
}

impl From<AVChromaLocation> for Location {
	fn from(value: AVChromaLocation) -> Self {
		match value {
			AVCHROMA_LOC_UNSPECIFIED => Location::Unspecified,
			AVCHROMA_LOC_LEFT => Location::Left,
			AVCHROMA_LOC_CENTER => Location::Center,
			AVCHROMA_LOC_TOPLEFT => Location::TopLeft,
			AVCHROMA_LOC_TOP => Location::Top,
			AVCHROMA_LOC_BOTTOMLEFT => Location::BottomLeft,
			AVCHROMA_LOC_BOTTOM => Location::Bottom,
			AVCHROMA_LOC_NB => Location::Unspecified,
		}
	}
}

impl Into<AVChromaLocation> for Location {
	fn into(self) -> AVChromaLocation {
		match self {
			Location::Unspecified => AVCHROMA_LOC_UNSPECIFIED,
			Location::Left => AVCHROMA_LOC_LEFT,
			Location::Center => AVCHROMA_LOC_CENTER,
			Location::TopLeft => AVCHROMA_LOC_TOPLEFT,
			Location::Top => AVCHROMA_LOC_TOP,
			Location::BottomLeft => AVCHROMA_LOC_BOTTOMLEFT,
			Location::Bottom => AVCHROMA_LOC_BOTTOM,
		}
	}
}
