/*
 *
 * Copyright (c) 2025.
 * All rights reserved.
 *
 */

use core::ops::Deref;

use noalloc_vec_rs::vec::Vec;

pub const END_CHAR: u8 = 0xC0;
pub const ESC_CHAR: u8 = 0xDB;
pub const ESC_END_CHAR: u8 = 0xDC;
pub const ESC_ESC_CHAR: u8 = 0xDD;

/// A SLIP encoder.
///
/// This struct provides a method to encode a packet using the SLIP protocol.
pub struct SlipEncoder;

impl SlipEncoder {
    /// Takes a reference to a Vec and encodes it in place.
    /// The Vec must have enough capacity to hold the encoded packet.
    ///
    /// # Arguments
    ///
    /// * `vec` - A reference to a Vec containing the packet to encode.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the packet was encoded successfully.
    /// * `Err(())` if the packet could not be encoded.
    #[allow(clippy::result_unit_err)]
    pub fn encode<const MAX_LENGTH: usize>(vec: &mut Vec<u8, MAX_LENGTH>) -> Result<(), ()> {
        // Begin the SLIP frame
        vec.insert(0, END_CHAR)?;

        let mut index = 1;
        while index < vec.len() {
            match vec[index] {
                END_CHAR => {
                    vec.insert(index, ESC_CHAR)?;
                    vec.write(index + 1, ESC_END_CHAR)?;
                    index += 2;
                }
                ESC_CHAR => {
                    vec.insert(index, ESC_CHAR)?;
                    vec.write(index + 1, ESC_ESC_CHAR)?;
                    index += 2;
                }
                _ => {
                    index += 1;
                }
            }
        }

        // End the SLIP frame
        vec.insert(vec.len(), END_CHAR)?;

        Ok(())
    }
}

/// The state of the SLIP decoder.
#[derive(Debug, Default, PartialEq)]
enum SlipDecoderState {
    /// The decoder is waiting for the start of a packet.
    #[default]
    Start,
    /// The decoder has reached the end of a packet.
    End,
    /// The decoder is appending bytes to the packet.
    Append,
    /// The decoder has encountered an escape character.
    Escape,
}

/// A SLIP decoder.
///
/// This struct provides methods to decode a packet using the SLIP protocol.
#[derive(Default)]
pub struct SlipDecoder<const MAX_LENGTH: usize> {
    /// The current state of the decoder.
    state: SlipDecoderState,
    /// The buffer containing the decoded packet.
    buffer: Vec<u8, MAX_LENGTH>,
}

impl<const MAX_LENGTH: usize> SlipDecoder<MAX_LENGTH> {
    /// Takes a byte and inserts it into the decoder.
    ///
    /// # Arguments
    ///
    /// * `value` - The byte to insert.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the byte was inserted successfully.
    /// * `Err(())` if the byte could not be inserted.
    #[allow(clippy::result_unit_err)]
    pub fn insert(&mut self, value: u8) -> Result<(), ()> {
        match self.state {
            SlipDecoderState::Start => {
                if value == END_CHAR {
                    self.state = SlipDecoderState::Append;
                }

                Ok(())
            }
            SlipDecoderState::Append => {
                match value {
                    END_CHAR => {
                        self.state = SlipDecoderState::End;
                    }
                    ESC_CHAR => {
                        self.state = SlipDecoderState::Escape;
                    }
                    _ => {
                        self.buffer.push(value)?;
                    }
                }

                Ok(())
            }
            SlipDecoderState::Escape => {
                self.state = SlipDecoderState::Append;

                match value {
                    ESC_END_CHAR => {
                        self.buffer.push(END_CHAR)?;

                        Ok(())
                    }
                    ESC_ESC_CHAR => {
                        self.buffer.push(ESC_CHAR)?;

                        Ok(())
                    }
                    _ => Err(()),
                }
            }
            SlipDecoderState::End => Err(()),
        }
    }

    /// Resets the decoder to its initial state.
    pub fn reset(&mut self) {
        self.state = SlipDecoderState::Start;
        self.buffer.clear();
    }

    /// Returns true if the decoder has reached the end of a packet.
    ///
    /// # Returns
    ///
    /// * `true` if the buffer is completed.
    /// * `false` otherwise.
    #[must_use]
    pub fn is_buffer_completed(&self) -> bool {
        self.state == SlipDecoderState::End
    }

    /// Returns a reference to the buffer containing the decoded packet.
    ///
    /// # Returns
    ///
    /// * A reference to the buffer.
    #[must_use]
    pub const fn get_buffer(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}

/// Deref to get the internal buffer.
impl<const MAX_LENGTH: usize> Deref for SlipDecoder<MAX_LENGTH> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.get_buffer()
    }
}

#[cfg(test)]
mod tests {
    use crate::slip::END_CHAR;
    use crate::slip::ESC_CHAR;
    use crate::slip::ESC_END_CHAR;
    use crate::slip::ESC_ESC_CHAR;
    use crate::slip::SlipDecoder;
    use crate::slip::SlipDecoderState;
    use crate::slip::SlipEncoder;
    use noalloc_vec_rs::vec::Vec;

    #[test]
    fn test_encode() {
        let mut array = Vec::<u8, 12>::from([0x00, 0x01, 0x02, 0x03]);

        let result = SlipEncoder::encode(&mut array);

        assert!(result.is_ok());
        assert_eq!(*array, [END_CHAR, 0x00, 0x01, 0x02, 0x03, END_CHAR]);
    }

    #[test]
    fn test_encode_empty() {
        let mut array = Vec::<u8, 12>::new();

        let result = SlipEncoder::encode(&mut array);

        assert!(result.is_ok());
        assert_eq!(*array, [END_CHAR, END_CHAR]);
    }

    #[test]
    fn test_encode_with_escape_characters() {
        let mut array = Vec::<u8, 12>::from([END_CHAR, ESC_CHAR, ESC_END_CHAR, ESC_ESC_CHAR]);

        let result = SlipEncoder::encode(&mut array);

        assert!(result.is_ok());
        assert_eq!(
            *array,
            [
                END_CHAR,
                ESC_CHAR,
                ESC_END_CHAR,
                ESC_CHAR,
                ESC_ESC_CHAR,
                ESC_END_CHAR,
                ESC_ESC_CHAR,
                END_CHAR
            ]
        );
    }

    #[test]
    fn test_decode() {
        let mut slip_decoder = SlipDecoder::<1>::default();

        assert_eq!(slip_decoder.state, SlipDecoderState::Start);

        let result = slip_decoder.insert(END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(0x00);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::End);

        assert!(slip_decoder.is_buffer_completed());

        assert_eq!(slip_decoder.get_buffer(), &[0x00]);
    }

    #[test]
    fn test_decode_with_escape_characters() {
        let mut slip_decoder = SlipDecoder::<6>::default();

        assert_eq!(slip_decoder.state, SlipDecoderState::Start);

        let result = slip_decoder.insert(END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(ESC_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Escape);

        let result = slip_decoder.insert(ESC_END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(ESC_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Escape);

        let result = slip_decoder.insert(ESC_ESC_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::End);

        assert!(slip_decoder.is_buffer_completed());

        assert_eq!(slip_decoder.get_buffer(), &[END_CHAR, ESC_CHAR]);
    }

    #[test]
    fn test_decode_with_bad_escape_character() {
        let mut slip_decoder = SlipDecoder::<1>::default();

        assert_eq!(slip_decoder.state, SlipDecoderState::Start);

        let result = slip_decoder.insert(END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(ESC_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Escape);

        let result = slip_decoder.insert(0x00);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_empty() {
        let mut slip_decoder = SlipDecoder::<0>::default();

        assert_eq!(slip_decoder.state, SlipDecoderState::Start);

        let result = slip_decoder.insert(END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::End);

        assert!(slip_decoder.is_buffer_completed());

        assert_eq!(slip_decoder.get_buffer(), &[]);
    }

    #[test]
    fn test_decode_and_reset() {
        let mut slip_decoder = SlipDecoder::<1>::default();

        assert_eq!(slip_decoder.state, SlipDecoderState::Start);

        let result = slip_decoder.insert(END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(0x00);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::End);

        assert!(slip_decoder.is_buffer_completed());

        assert_eq!(slip_decoder.get_buffer(), &[0x00]);

        slip_decoder.reset();
        assert_eq!(slip_decoder.state, SlipDecoderState::Start);
        assert!(!slip_decoder.is_buffer_completed());
    }

    #[test]
    fn test_decode_with_not_enough_space() {
        let mut slip_decoder = SlipDecoder::<1>::default();

        assert_eq!(slip_decoder.state, SlipDecoderState::Start);

        let result = slip_decoder.insert(END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(0x00);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(0x00);
        assert!(result.is_err());
    }
}
