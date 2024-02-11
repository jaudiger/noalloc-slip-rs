/*
 *
 * Copyright (c) 2024.
 * All rights reserved.
 *
 */

use crate::vec::Vec;

const END_CHAR: u8 = 0xC0;
const ESC_CHAR: u8 = 0xDB;
const ESC_END_CHAR: u8 = 0xDC;
const ESC_ESC_CHAR: u8 = 0xDD;

pub struct SlipEncoder<const MAX_LENGTH: usize>(Vec<u8, MAX_LENGTH>);

impl<const MAX_LENGTH: usize> SlipEncoder<MAX_LENGTH> {
    #[inline]
    #[must_use]
    pub const fn new(array: Vec<u8, MAX_LENGTH>) -> Self {
        Self(array)
    }

    pub fn encode(mut self) -> Result<Vec<u8, MAX_LENGTH>, ()> {
        // Begin the SLIP frame
        self.0.insert(0, END_CHAR)?;

        let mut index = 1;
        while index < self.0.len() {
            match self.0[index] {
                END_CHAR => {
                    self.0.insert(index, ESC_CHAR)?;
                    self.0.write(index + 1, ESC_END_CHAR)?;
                    index += 2;
                }
                ESC_CHAR => {
                    self.0.insert(index, ESC_CHAR)?;
                    self.0.write(index + 1, ESC_ESC_CHAR)?;
                    index += 2;
                }
                _ => {
                    index += 1;
                }
            }
        }

        // End the SLIP frame
        self.0.insert(self.0.len(), END_CHAR)?;

        Ok(self.0)
    }
}

#[derive(Debug, Default, PartialEq)]
enum SlipDecoderState {
    #[default]
    Start,
    End,
    Append,
    Escape,
}

#[derive(Default)]
pub struct SlipDecoder<const MAX_LENGTH: usize> {
    state: SlipDecoderState,
    buffer: Vec<u8, MAX_LENGTH>,
}

impl<const MAX_LENGTH: usize> SlipDecoder<MAX_LENGTH> {
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
                    }
                    ESC_ESC_CHAR => {
                        self.buffer.push(ESC_CHAR)?;
                    }
                    _ => {
                        self.buffer.push(ESC_CHAR)?;
                        self.buffer.push(value)?;
                    }
                }

                Ok(())
            }
            SlipDecoderState::End => Err(()),
        }
    }

    pub fn reset(&mut self) {
        self.state = SlipDecoderState::Start;
        self.buffer.clear();
    }

    #[must_use]
    pub fn get_buffer(&self) -> Option<&[u8]> {
        if self.state == SlipDecoderState::End {
            Some(&self.buffer)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::slip::SlipDecoder;
    use crate::slip::SlipDecoderState;
    use crate::slip::SlipEncoder;
    use crate::slip::END_CHAR;
    use crate::slip::ESC_CHAR;
    use crate::slip::ESC_END_CHAR;
    use crate::slip::ESC_ESC_CHAR;
    use crate::vec::Vec;

    #[test]
    fn test_encode() {
        let array = Vec::<u8, 12>::from([0x00, 0x01, 0x02, 0x03]);
        let slip_encoder = SlipEncoder::new(array);

        let result = slip_encoder.encode();

        assert!(result.is_ok());
        assert_eq!(
            *result.unwrap(),
            [END_CHAR, 0x00, 0x01, 0x02, 0x03, END_CHAR]
        );
    }

    #[test]
    fn test_encode_empty() {
        let array = Vec::<u8, 12>::new();
        let slip_encoder = SlipEncoder::new(array);

        let result = slip_encoder.encode();

        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), [END_CHAR, END_CHAR]);
    }

    #[test]
    fn test_encode_with_escape_characters() {
        let array = Vec::<u8, 12>::from([END_CHAR, ESC_CHAR, ESC_END_CHAR, ESC_ESC_CHAR]);
        let slip_encoder = SlipEncoder::new(array);

        let result = slip_encoder.encode();

        assert!(result.is_ok());
        assert_eq!(
            *result.unwrap(),
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

        let buffer: Option<&[u8]> = slip_decoder.get_buffer();
        let expected_buffer: [u8; 1] = [0x00];
        assert_eq!(buffer, Some(&expected_buffer as &[u8]));
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

        let result = slip_decoder.insert(ESC_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Escape);

        let result = slip_decoder.insert(ESC_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(ESC_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Escape);

        let result = slip_decoder.insert(0);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::Append);

        let result = slip_decoder.insert(END_CHAR);
        assert!(result.is_ok());
        assert_eq!(slip_decoder.state, SlipDecoderState::End);

        let buffer: Option<&[u8]> = slip_decoder.get_buffer();
        let expected_buffer: [u8; 6] = [END_CHAR, ESC_CHAR, ESC_CHAR, ESC_CHAR, ESC_CHAR, 0x00];
        assert_eq!(buffer, Some(&expected_buffer as &[u8]));
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

        let buffer: Option<&[u8]> = slip_decoder.get_buffer();
        let expected_buffer: [u8; 0] = [];
        assert_eq!(buffer, Some(&expected_buffer as &[u8]));
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

        let buffer: Option<&[u8]> = slip_decoder.get_buffer();
        let expected_buffer: [u8; 1] = [0x00];
        assert_eq!(buffer, Some(&expected_buffer as &[u8]));

        slip_decoder.reset();
        assert_eq!(slip_decoder.state, SlipDecoderState::Start);
        assert_eq!(slip_decoder.get_buffer(), None);
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
