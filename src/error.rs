use std::{
    error,
    fmt,
    io,
};

use gif::DecodingError;

use gif_dispose::Error as GifDisposeError;

#[derive(Debug)]
pub enum ConvError {
    /// (user x, user y)
    HotspotOutOfMaxRange(u16, u16),
    /// (user x, user y, gif x, gif y)
    HotspotOutOfGifRange(u16, u16, u16, u16),
    /// User given path to .gif file.
    InvalidGifPath(String),
    InvalidAniExtension(String),
    InvalidHotspotDefinition,
    FailedAniEncoding,
    FailedPngEncoding,
    FailedFileCreation,
    DecodingError,
    GifDisposeError,
    UserInputError,
}

impl fmt::Display for ConvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConvError::HotspotOutOfMaxRange(x, y) => {
                write!(f, "Hotspot can't be above 256. Given coordinates: x: {}, y: {}.", x, y)
            }
            ConvError::HotspotOutOfGifRange(user_x, user_y, gif_x, gif_y) => {
                write!(f, "Hotspot can't be above x: {}, y: {}. Given coordinates: x: {}, y: {}.", gif_x, gif_y, user_x, user_y)
            }
            ConvError::InvalidGifPath(path) => {
                write!(f, "No .gif file found under {}.", path)
            }
            ConvError::InvalidAniExtension(path) => {
                write!(f, "{} doesn't have the .ani file extension.", path)
            }
            ConvError::InvalidHotspotDefinition => {
                write!(f, "Hotspot could not be parsed properly. Try the following syntax: `x:y`.")
            }
            ConvError::FailedAniEncoding => {
                write!(f, "Failed to encode to ani format correctly.")
            }
            ConvError::FailedPngEncoding => {
                write!(f, "Failed to encode to png format correctly.")
            }
            ConvError::FailedFileCreation => {
                write!(f, "Failed to create files.")
            }
            ConvError::DecodingError => {
                write!(f, "Failed to decode.")
            }
            ConvError::GifDisposeError => {
                write!(f, "Failed to dispose gif.")
            }
            ConvError::UserInputError => {
                write!(f, "Stopping conversion.")
            }
            #[allow(unreachable_patterns)]
            _ => { write!(f, "Unkown Error.") }
        }
    }
}

impl error::Error for ConvError {}

impl From<DecodingError> for ConvError {
    fn from(_: DecodingError) -> Self {
        Self::DecodingError
    }
}

impl From<GifDisposeError> for ConvError {
    fn from(_: GifDisposeError) -> Self {
        Self::GifDisposeError
    }
}

impl From<io::Error> for ConvError {
    fn from(_: io::Error) -> Self {
        Self::UserInputError
    }
}