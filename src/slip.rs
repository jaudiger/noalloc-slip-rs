/*
 *
 * Copyright (c) 2024.
 * All rights reserved.
 *
 */

use crate::vec::Vec;

const END: u8 = 0xC0;
const ESC: u8 = 0xDB;
const ESC_END: u8 = 0xDC;
const ESC_ESC: u8 = 0xDD;

pub struct SlipEncoder<const MAX_LENGTH: usize>(Vec<u8, MAX_LENGTH>);

impl<const MAX_LENGTH: usize> SlipEncoder<MAX_LENGTH> {
    #[inline]
    pub fn new(array: Vec<u8, MAX_LENGTH>) -> Self {
        Self { 0: array }
    }

    pub fn encode(mut self) -> Result<Vec<u8, MAX_LENGTH>, ()> {
        // Begin the SLIP frame
        self.0.insert(0, END)?;

        let mut index = 1;
        while index < self.0.len() {
            match self.0[index] {
                END => {
                    self.0.insert(index, ESC)?;
                    self.0.write(index + 1, ESC_END)?;
                    index += 2;
                }
                ESC => {
                    self.0.insert(index, ESC)?;
                    self.0.write(index + 1, ESC_ESC)?;
                    index += 2;
                }
                _ => {
                    index += 1;
                }
            }
        }

        // End the SLIP frame
        self.0.insert(self.0.len(), END)?;

        Ok(self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::slip::SlipEncoder;
    use crate::slip::END;
    use crate::slip::ESC;
    use crate::slip::ESC_END;
    use crate::slip::ESC_ESC;
    use crate::vec::Vec;

    #[test]
    fn test_encode() {
        let array = Vec::<u8, 12>::from([0x00, 0x01, 0x02, 0x03]);
        let slip = SlipEncoder::new(array);

        let result = slip.encode();

        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), [END, 0x00, 0x01, 0x02, 0x03, END]);
    }

    #[test]
    fn test_encode_empty() {
        let array = Vec::<u8, 12>::new();
        let slip = SlipEncoder::new(array);

        let result = slip.encode();

        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), [END, END]);
    }

    #[test]
    fn test_encode_with_escape_characters() {
        let array = Vec::<u8, 12>::from([END, ESC, ESC_END, ESC_ESC]);
        let slip = SlipEncoder::new(array);

        let result = slip.encode();

        assert!(result.is_ok());
        assert_eq!(
            *result.unwrap(),
            [END, ESC, ESC_END, ESC, ESC_ESC, ESC_END, ESC_ESC, END]
        );
    }
}
