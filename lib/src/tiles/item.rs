use super::codes::Code;
use crate::error::Error;
use std::convert::TryFrom;
use std::fmt::Debug;

/// common trait for [`Tile`](crate::Tile), [`Letter`](crate::Letter), [`Cell`](crate::Cell)
pub trait Item:
    Debug + Clone + Copy + PartialEq + Default + Into<Code> + TryFrom<Code, Error = Error>
{
    fn code(&self) -> Code;
}
