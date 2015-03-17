// Delta encoding algorithm
use std::io::Read;
use std::str as Str;

use reader::MyReaderExt;

// TODO: Ensure this isn't dependent on the std::old_io
// implementation of Reader.

pub fn patch_file(source_path: &str, delta_path: &str) {
    use std::fs::File;

    let mut source_file = File::open(source_path).unwrap();
    let mut source_contents = Vec::new();

    let mut delta_file = File::open(delta_path).unwrap();
    let mut delta_contents = Vec::new();

    let _ = source_file.read_to_end(&mut source_contents);
    let _ = delta_file.read_to_end(&mut delta_contents);

    let mut patcher = DeltaPatcher::new(&source_contents[..], &delta_contents[..]);
    let res = patcher.run_to_end();
    println!("{}", Str::from_utf8(&res[..]).unwrap());
}

#[derive(Debug)]
struct DeltaHeader {
    source_len: usize,
    target_len: usize,
    get_offset: usize,
}

impl DeltaHeader {
    fn new(delta: &mut &[u8]) -> DeltaHeader {
        let (source, bytes_s) = DeltaHeader::decode_size(delta);
        let (target, bytes_t) = DeltaHeader::decode_size(delta);

        DeltaHeader {
            source_len: source,
            target_len: target,
            get_offset: bytes_s + bytes_t,
        }
    }

    fn decode_size(delta: &mut &[u8]) -> (usize, usize) {
        let mut byte = 0x80;
        let mut size = 0;
        let mut count = 0;

        while (byte & 0x80) > 0 {
            byte = MyReaderExt::read_byte(delta).unwrap() as usize;
            size += (byte & 127) << (7 * count);

            count += 1;
        }
        return (size, count);
    }
}

#[derive(Debug)]
enum DeltaOp {
    Insert(usize),
    Copy(usize, usize)
}

struct DeltaPatcher<'a> {
    source: &'a [u8],
    delta: &'a [u8],
    target_len: usize,
}

impl<'a> DeltaPatcher<'a> {
    pub fn new(source: &'a [u8], mut delta: &'a [u8]) -> Self {
        let header = DeltaHeader::new(&mut delta);

        DeltaPatcher {
            source: source,
            delta: delta,
            target_len: header.target_len
        }
    }

    fn run_to_end(&mut self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.target_len);
        loop {
            if let Some(buf) = self.next() {
                result.push_all(&buf[..]);
            } else {
                break;
            }
        }
        result
    }

    fn next(&mut self) -> Option<Vec<u8>> {
        if self.delta.len() > 0 {
            let command = self.next_command();
            let result = self.run_command(command);
            Some(result)
        } else {
            None
        }
    }

    fn next_command(&mut self) -> DeltaOp {
        let cmd = MyReaderExt::read_byte(&mut self.delta).unwrap();

        if cmd & 128 > 0 {
            let mut offset = 0usize;
            let mut shift = 0usize;
            let mut length = 0usize;

            // Read the offset to copy from
            for &mask in [0x01, 0x02, 0x04, 0x08].iter() {
                if cmd & mask > 0 {
                    let byte = MyReaderExt::read_byte(&mut self.delta).unwrap() as u64;
                    offset += (byte as usize) << shift;
                }
                shift += 8;
            }

            // Read the length of the copy
            shift = 0;
            for &mask in [0x10, 0x20, 0x40].iter() {
                if cmd & mask > 0 {
                    let byte = MyReaderExt::read_byte(&mut self.delta).unwrap() as u64;
                    length += (byte << shift) as usize;
                }
                shift += 8;
            }
            DeltaOp::Copy(offset, length)
        } else {
            DeltaOp::Insert(cmd as usize)
        }
    }

    fn run_command(&mut self, command: DeltaOp) -> Vec<u8> {
        match command {
            DeltaOp::Copy(start, length) => {
                let end = start + length;
                let mut buf = Vec::with_capacity(length);
                buf.push_all(&self.source[start..end]);
                buf
            },
            DeltaOp::Insert(length) => {
                // TODO: Shouldn't have to do another allocation here in the read
                let items = MyReaderExt::read_exact(&mut self.delta, length as u64).unwrap();
                let mut buf = Vec::with_capacity(length);
                buf.push_all(&items[..]);
                buf
            }
        }
    }
}

