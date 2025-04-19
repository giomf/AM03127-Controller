#![allow(dead_code)]

pub mod formatting;

use core::fmt::{self, Display, Write};
use heapless::String;

use super::{DEFAULT_ID, DEFAULT_LINE, DEFAULT_PAGE, STRING_SIZE, wrap_command};

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

/// Enum representing different speed levels and actions.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitingModeAndSpeed {
    /// Display Blinking while waiting
    FastestBlinking,
    /// Display stay steady while waiting
    #[default]
    FastestNormal,
    /// Play pre-defined song 1
    FastestSong1,
    /// Play pre-defined song 2
    FastestSong2,
    /// Play pre-defined song 3
    FastestSong3,

    /// Display Blinking while waiting
    MiddleFastBlinking,
    /// Display stay steady while waiting
    MiddleFastNormal,
    /// Play pre-defined song 1
    MiddleFastSong1,
    /// Play pre-defined song 2
    MiddleFastSong2,
    /// Play pre-defined song 3
    MiddleFastSong3,

    /// Display Blinking while waiting
    MiddleSlowBlinking,
    /// Display stay steady while waiting
    MiddleSlowNormal,
    /// Play pre-defined song 1
    MiddleSlowSong1,
    /// Play pre-defined song 2
    MiddleSlowSong2,
    /// Play pre-defined song 3
    MiddleSlowSong3,

    /// Display Blinking while waiting
    SlowestBlinking,
    /// Display stay steady while waiting
    SlowestNormal,
    /// Play pre-defined song 1
    SlowestSong1,
    /// Play pre-defined song 2
    SlowestSong2,
    /// Play pre-defined song 3
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

#[derive(Debug, Clone)]
pub struct PageContent {
    id: u8,
    line: u8,
    page: char,
    leading: Leading,
    lagging: Lagging,
    waiting_mode_and_speed: WaitingModeAndSpeed,
    message: String<STRING_SIZE>,
}

impl PageContent {
    pub fn id(mut self, id: u8) -> Self {
        self.id = id;
        self
    }
    pub fn line(mut self, line: u8) -> Self {
        self.line = line;
        self
    }
    pub fn page(mut self, page: char) -> Self {
        self.page = page;
        self
    }
    pub fn leading(mut self, leading: Leading) -> Self {
        self.leading = leading;
        self
    }
    pub fn lagging(mut self, lagging: Lagging) -> Self {
        self.lagging = lagging;
        self
    }
    pub fn waiting_mode_and_speed(mut self, waiting_mode_and_speed: WaitingModeAndSpeed) -> Self {
        self.waiting_mode_and_speed = waiting_mode_and_speed;
        self
    }
    pub fn message(mut self, message: &str) -> Self {
        self.message.clear();
        let _ = self.message.push_str(message);
        self
    }

    pub fn command(&self) -> String<STRING_SIZE> {
        let mut command = String::<STRING_SIZE>::new();
        write!(
            &mut command,
            "<L{}><P{}><F{}><M{}><WA><F{}>",
            self.line, self.page, self.leading, self.waiting_mode_and_speed, self.lagging
        )
        .unwrap();

        // Append the processed message
        let processed = Self::replace_european_character(&self.message);
        let _ = command.push_str(&processed);

        wrap_command(self.id, &command)
    }

    fn replace_european_character(message: &str) -> String<STRING_SIZE> {
        let mut result = String::<STRING_SIZE>::new();
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

impl Default for PageContent {
    fn default() -> Self {
        Self {
            id: DEFAULT_ID,
            page: DEFAULT_PAGE,
            line: DEFAULT_LINE,
            leading: Default::default(),
            lagging: Default::default(),
            waiting_mode_and_speed: Default::default(),
            message: String::new(),
        }
    }
}
