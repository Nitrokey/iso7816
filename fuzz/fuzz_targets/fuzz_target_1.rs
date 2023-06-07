#![no_main]

use libfuzzer_sys::fuzz_target;

use arbitrary::Arbitrary;
use iso7816::command::{class, Command, CommandBuilder, CommandView};

use std::convert::Infallible;
use std::iter::repeat;
use std::ops::Deref;

struct WriteMock {
    buffer: [u8; 4096],
    written: usize,
    capacity: usize,
}

impl Deref for WriteMock {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.buffer[..self.written]
    }
}

impl iso7816::command::Writer for WriteMock {
    type Error = Infallible;
    fn write(&mut self, data: &[u8]) -> Result<usize, Infallible> {
        let available = self.capacity - self.written;
        let written = available.min(data.len());
        self.buffer[self.written..][..written].copy_from_slice(data);
        self.written += written;
        Ok(written)
    }

    fn remaining_len(&self) -> usize {
        self.capacity - self.written
    }
}

#[derive(Debug, Arbitrary)]
struct Input<'a> {
    class: u8,
    instruction: u8,
    p1: u8,
    p2: u8,
    le: u16,
    buf_len: usize,
    buf_lens: Vec<usize>,
    supports_extended: bool,
    data: &'a [u8],
}

fuzz_target!(|data: Input| {
    let Input {
        class,
        instruction,
        p1,
        p2,
        le,
        buf_len,
        buf_lens,
        supports_extended,
        data,
    } = data;
    if class == 0b11101111 {
        // pathological class that can't be chained because it makes it a 0xFF
        return;
    }
    let Ok(class) = class::Class::try_from(class) else {
        return;
    };
    let ins = instruction.into();
    let command = CommandBuilder::new(class, ins, p1, p2, data, le);

    // Test for the length information
    {
        command.clone().serialize_to_vec();
    }

    let mut buffer = WriteMock {
        buffer: [0; 4096],
        written: 0,
        capacity: buf_len.min(4096).max(128),
    };

    match command
        .clone()
        .serialize_into(&mut buffer, supports_extended)
        .unwrap()
    {
        Ok(()) => {
            // dbg!(&*buffer, buffer.len());
            let view = CommandView::try_from(&*buffer).unwrap();
            if !supports_extended {
                assert!(view.data().len() <= 256);
                assert!(!view.extended());
                // Without extended support, le is truncated to 256 at max, and the response will come with command chaining
                let command = CommandBuilder::new(class, ins, p1, p2, data, le.min(256));
                assert_eq!(command, view);
            } else {
                assert_eq!(view, command);
            }
        }

        Err(mut rem) => {
            let len = buffer.len();
            // dbg!(&*buffer, buffer.len());
            let mut parsed = Command::<4096>::try_from(&buffer[..len]).unwrap();
            if !supports_extended {
                assert!(parsed.data().len() <= 255);
                assert!(!parsed.extended());
            }
            // Loop with arbitrary buflens forever
            for buflen in repeat(buf_lens.iter().chain([&128])).flatten() {
                let mut buffer = WriteMock {
                    buffer: [0; 4096],
                    written: 0,
                    capacity: (*buflen).min(4096).max(128),
                };
                match rem.serialize_into(&mut buffer, supports_extended).unwrap() {
                    Ok(()) => {
                        // dbg!(&*buffer, buffer.len());
                        let view = CommandView::try_from(&*buffer).unwrap();
                        if !supports_extended {
                            assert!(view.data().len() <= 255);
                            assert!(!view.extended());
                        }
                        parsed.extend_from_command_view(view).unwrap();
                        if supports_extended {
                            assert_eq!(command, parsed.as_view());
                        } else {
                            // Without extended support, le is truncated to 255 at max, and the response will come with command chaining
                            let command =
                                CommandBuilder::new(class, ins, p1, p2, data, le.min(256));
                            assert_eq!(command, parsed.as_view());
                        }
                        return;
                    }
                    Err(new_rem) => {
                        rem = new_rem;

                        let view = CommandView::try_from(&*buffer).unwrap();
                        if !supports_extended {
                            assert!(view.data().len() <= 255);
                            assert!(!view.extended());
                        }
                        parsed.extend_from_command_view(view).unwrap();
                    }
                }
            }
        }
    }
    // fuzzed code goes here
});
