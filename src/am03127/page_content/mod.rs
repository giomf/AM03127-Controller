#![allow(dead_code)]

pub mod formatting;

use core::fmt::{self, Display};
use heapless::String;
use serde::{Deserialize, Serialize};

use crate::server::dto::PageDto;

use super::{CommandAble, DEFAULT_LINE, DEFAULT_PAGE, MESSAGE_STRING_SIZE};

/// Leading effects for displaying content on the LED panel
///
/// These effects control how content appears on the panel when it is first displayed.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Leading {
    ///  8 pixel width display block will be moved from right to left one by one
    BlockMove,
    /// Image will be shown one line by one line from top to bottom
    CurtainDown,
    /// Image will be shown one line by one line from bottom to top
    CurtainUp,
    /// Previous screen will be kept
    Hold,
    /// Image will be immediately appeared
    #[default]
    Immediate,
    /// Pen writing 'Amplus'
    PenAmplus,
    /// Pen writing 'Hello World'
    PenHelloWorld,
    /// Pen writing 'Welcome'
    PenWelcome,
    /// Random pixels will be appeared to build the image
    Random,
    /// Image will be scrolled from top to bottom
    ScrollDown,
    /// Image will be scrolled from right to left
    ScrollLeft,
    /// Image will be scrolled from left to right
    ScrollRight,
    /// Image will be scrolled from bottom to top
    ScrollUp,
    /// Pixels will be dropped down from top and stack up to build the image
    Snow,
    /// A blank diagonal line will be scrolling on the image
    Twinkle,
    /// Image will be shown from top and bottom to center one line by one line
    Vclose,
    /// Image will be shown from center to top and bottom one line by one line
    Vopen,
    /// Image will be shown from center and extend to 4 sides
    Xopen,
}

impl Display for Leading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let character = match self {
            Leading::Immediate => 'A',
            Leading::Xopen => 'B',
            Leading::CurtainUp => 'C',
            Leading::CurtainDown => 'D',
            Leading::ScrollLeft => 'E',
            Leading::ScrollRight => 'F',
            Leading::Vopen => 'G',
            Leading::Vclose => 'H',
            Leading::ScrollUp => 'I',
            Leading::ScrollDown => 'J',
            Leading::Hold => 'K',
            Leading::Snow => 'L',
            Leading::Twinkle => 'M',
            Leading::BlockMove => 'N',
            Leading::Random => 'P',
            Leading::PenHelloWorld => 'Q',
            Leading::PenWelcome => 'R',
            Leading::PenAmplus => 'S',
        };

        write!(f, "{character}")
    }
}
/// Lagging effects for content on the LED panel
///
/// These effects control how content disappears from the panel when it is removed.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lagging {
    /// Image will be disappeared one line by one line from top to bottom
    CurtainDown,
    /// Image will be disappeared one line by one line from bottom to top
    CurtainUp,
    #[default]
    /// Screen will be kept
    Hold,
    /// Image will be immediately disappeared
    Immediate,
    /// Image will be scrolled from bottom to top and disappeared
    ScrollDown,
    /// Image will be scrolled from right to left and disappeared
    ScrollLeft,
    /// Image will be scrolled from right to left and disappeared
    ScrollRight,
    /// Image will be scrolled from bottom to top and disappeared
    ScrollUp,
    /// Image will be disappeared from top and bottom to center one line by one line
    Vclose,
    /// Image will be disappeared from center to top and bottom one line by one line
    Vopen,
    /// Image will be disappeared from center and extend to 4 sides
    Xopen,
}

impl Display for Lagging {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let character = match self {
            Lagging::Immediate => 'A',
            Lagging::Xopen => 'B',
            Lagging::CurtainUp => 'C',
            Lagging::CurtainDown => 'D',
            Lagging::ScrollLeft => 'E',
            Lagging::ScrollRight => 'F',
            Lagging::Vopen => 'G',
            Lagging::Vclose => 'H',
            Lagging::ScrollUp => 'I',
            Lagging::ScrollDown => 'J',
            Lagging::Hold => 'K',
        };

        write!(f, "{}", character)
    }
}

/// Waiting modes and speeds for content on the LED panel
///
/// These settings control how content behaves while it is being displayed,
/// including speed of transitions and special effects like blinking or playing sounds.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitingModeAndSpeed {
    /// Display Blinking while waiting (fastest speed)
    FastestBlinking,
    /// Display stay steady while waiting (fastest speed)
    #[default]
    FastestNormal,
    /// Play pre-defined song 1 (fastest speed)
    FastestSong1,
    /// Play pre-defined song 2 (fastest speed)
    FastestSong2,
    /// Play pre-defined song 3 (fastest speed)
    FastestSong3,

    /// Display Blinking while waiting (middle-fast speed)
    MiddleFastBlinking,
    /// Display stay steady while waiting (middle-fast speed)
    MiddleFastNormal,
    /// Play pre-defined song 1 (middle-fast speed)
    MiddleFastSong1,
    /// Play pre-defined song 2 (middle-fast speed)
    MiddleFastSong2,
    /// Play pre-defined song 3 (middle-fast speed)
    MiddleFastSong3,

    /// Display Blinking while waiting (middle-slow speed)
    MiddleSlowBlinking,
    /// Display stay steady while waiting (middle-slow speed)
    MiddleSlowNormal,
    /// Play pre-defined song 1 (middle-slow speed)
    MiddleSlowSong1,
    /// Play pre-defined song 2 (middle-slow speed)
    MiddleSlowSong2,
    /// Play pre-defined song 3 (middle-slow speed)
    MiddleSlowSong3,

    /// Display Blinking while waiting (slowest speed)
    SlowestBlinking,
    /// Display stay steady while waiting (slowest speed)
    SlowestNormal,
    /// Play pre-defined song 1 (slowest speed)
    SlowestSong1,
    /// Play pre-defined song 2 (slowest speed)
    SlowestSong2,
    /// Play pre-defined song 3 (slowest speed)
    SlowestSong3,
}

impl Display for WaitingModeAndSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let character = match self {
            WaitingModeAndSpeed::FastestNormal => 'A',
            WaitingModeAndSpeed::FastestBlinking => 'B',
            WaitingModeAndSpeed::FastestSong1 => 'C',
            WaitingModeAndSpeed::FastestSong2 => 'D',
            WaitingModeAndSpeed::FastestSong3 => 'E',

            WaitingModeAndSpeed::MiddleFastNormal => 'Q',
            WaitingModeAndSpeed::MiddleFastBlinking => 'R',
            WaitingModeAndSpeed::MiddleFastSong1 => 'S',
            WaitingModeAndSpeed::MiddleFastSong2 => 'T',
            WaitingModeAndSpeed::MiddleFastSong3 => 'U',

            WaitingModeAndSpeed::MiddleSlowNormal => 'a',
            WaitingModeAndSpeed::MiddleSlowBlinking => 'b',
            WaitingModeAndSpeed::MiddleSlowSong1 => 'c',
            WaitingModeAndSpeed::MiddleSlowSong2 => 'd',
            WaitingModeAndSpeed::MiddleSlowSong3 => 'e',

            WaitingModeAndSpeed::SlowestNormal => 'q',
            WaitingModeAndSpeed::SlowestBlinking => 'r',
            WaitingModeAndSpeed::SlowestSong1 => 's',
            WaitingModeAndSpeed::SlowestSong2 => 't',
            WaitingModeAndSpeed::SlowestSong3 => 'u',
        };

        write!(f, "{}", character)
    }
}

/// Represents a page of content for the LED panel
///
/// A page contains text content and display settings that control
/// how the content appears, behaves, and disappears on the panel.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Page {
    /// Line number (usually 1)
    line: u8,
    /// Page ID (A-Z)
    page: char,
    /// Effect for how the page appears
    pub leading: Leading,
    /// Effect for how the page disappears
    pub lagging: Lagging,
    /// Speed and behavior while the page is displayed
    pub waiting_mode_and_speed: WaitingModeAndSpeed,
    /// Text content of the page
    pub message: String<MESSAGE_STRING_SIZE>,
}

impl CommandAble for Page {}

impl Page {
    /// Sets the line number for the page
    ///
    /// # Arguments
    /// * `line` - The line number
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn line(mut self, line: u8) -> Self {
        self.line = line;
        self
    }
    
    /// Sets the page ID
    ///
    /// # Arguments
    /// * `page` - The page ID (A-Z)
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn page(mut self, page: char) -> Self {
        self.page = page;
        self
    }
    
    /// Sets the leading effect for the page
    ///
    /// # Arguments
    /// * `leading` - The leading effect
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn leading(mut self, leading: Leading) -> Self {
        self.leading = leading;
        self
    }
    
    /// Sets the lagging effect for the page
    ///
    /// # Arguments
    /// * `lagging` - The lagging effect
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn lagging(mut self, lagging: Lagging) -> Self {
        self.lagging = lagging;
        self
    }
    
    /// Sets the waiting mode and speed for the page
    ///
    /// # Arguments
    /// * `waiting_mode_and_speed` - The waiting mode and speed
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn waiting_mode_and_speed(mut self, waiting_mode_and_speed: WaitingModeAndSpeed) -> Self {
        self.waiting_mode_and_speed = waiting_mode_and_speed;
        self
    }
    
    /// Sets the message content for the page
    ///
    /// # Arguments
    /// * `message` - The message text
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn message(mut self, message: &str) -> Self {
        self.message.clear();
        let _ = self.message.push_str(message);
        self
    }

    /// Creates a Page from a DTO with a specific ID
    ///
    /// # Arguments
    /// * `page` - The page ID
    /// * `dto` - The PageDto containing page data
    ///
    /// # Returns
    /// * A new Page instance
    pub fn from_dto_with_id(page: char, dto: PageDto) -> Self {
        Self::default()
            .page(page)
            .leading(dto.leading)
            .lagging(dto.lagging)
            .waiting_mode_and_speed(dto.waiting_mode_and_speed)
            .message(&dto.text)
    }

    /// Replaces European characters with their panel-specific codes
    ///
    /// # Arguments
    /// * `message` - The message text with possible European characters
    ///
    /// # Returns
    /// * A new string with European characters replaced by panel codes
    fn replace_european_character(message: &str) -> String<MESSAGE_STRING_SIZE> {
        let mut result = String::<MESSAGE_STRING_SIZE>::new();
        for c in message.chars() {
            match c {
                'ü' => result.push_str("<U7C>").unwrap_or(()),
                'Ü' => result.push_str("<U5C>").unwrap_or(()),
                'ä' => result.push_str("<U64>").unwrap_or(()),
                'Ä' => result.push_str("<U44>").unwrap_or(()),
                'ö' => result.push_str("<U76>").unwrap_or(()),
                'Ö' => result.push_str("<U56>").unwrap_or(()),
                'ß' => result.push_str("<U5F>").unwrap_or(()),
                _ => result.push(c).unwrap_or(()),
            }
        }
        result
    }
}

impl Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = Self::replace_european_character(&self.message);
        write!(
            f,
            "<L{}><P{}><F{}><M{}><WA><F{}>{}",
            self.line, self.page, self.leading, self.waiting_mode_and_speed, self.lagging, message
        )
    }
}

impl Default for Page {
    fn default() -> Self {
        Self {
            page: DEFAULT_PAGE,
            line: DEFAULT_LINE,
            leading: Default::default(),
            lagging: Default::default(),
            waiting_mode_and_speed: Default::default(),
            message: String::new(),
        }
    }
}
