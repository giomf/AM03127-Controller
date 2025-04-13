#![allow(dead_code)]

use core::fmt::{self, Display};

/// Font size
pub enum Font {
    /// Normal size (5x7)
    Normal,
    /// Bold Size (6x7)
    Bold,
    /// Narrow Size (4x7)
    Narrow,
    /// Large Size (7x13), for 16 pixel height or more LED display only
    Large,
    /// Long Size (5x8), only for height more than 7 pixels
    Long,
}

impl Display for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let command = match self {
            Font::Normal => "<AA>",
            Font::Bold => "<AB>",
            Font::Narrow => "<AC>",
            Font::Large => "<AD>",
            Font::Long => "<AE>",
        };

        write!(f, "{command}")
    }
}

pub struct ColumnStart(pub u8);

impl Display for ColumnStart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<N{:02X}>", self.0)
    }
}

pub enum Clock {
    /// Date in format [DD/MM/YY]
    Date,
    /// Time in format [hh:mm]
    Time,
}

impl Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let command = match self {
            Clock::Date => "<KD>",
            Clock::Time => "<KT>",
        };

        write!(f, "{command}")
    }
}
