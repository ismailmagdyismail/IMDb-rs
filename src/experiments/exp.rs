use std::io::{BufReader, BufWriter, Read, Write};

const KEY_HEADER_SIZE: u32 = 4;
const VALUE_HEADER_SIZE: u32 = 4;
const HEADER_SIZE: u32 = KEY_HEADER_SIZE + VALUE_HEADER_SIZE;

fn seperate_index_from_record() {}

fn write_index_with_record() {
    let file = std::fs::File::create("index_data.bin").unwrap();
    let iterations: u32 = 100_000_000;
    let val = "ismail";
    let len = HEADER_SIZE + 4 + val.len() as u32; // header, key , value
    let mut buffer: Vec<u8> = vec![b'0'; len as usize];
    let mut buf_writer = BufWriter::new(file);
    for i in 0..iterations {
        // serialize key header
        let key_len_header = &mut buffer.as_mut_slice()[0..4];
        let ser_key_header = (i.to_ne_bytes().len() as u32).to_ne_bytes();
        key_len_header.copy_from_slice(&ser_key_header);

        // serialize value header
        let value_header_len = &mut buffer.as_mut_slice()[4..8];
        let ser_value_header: [u8; 4] = (val.len() as u32).to_ne_bytes();
        value_header_len.copy_from_slice(&ser_value_header);

        // serialize kv pairs
        let key_bytes_count = i.to_ne_bytes().len();
        let key_slice = &mut buffer.as_mut_slice()[8..(8 + key_bytes_count)];
        let key_ser_bytes = i.to_ne_bytes();
        key_slice.copy_from_slice(&key_ser_bytes);

        // serialize value
        let value_bytes_count = val.len();
        let val_offset: usize = 8 + key_bytes_count;
        let value_slice = &mut buffer.as_mut_slice()[val_offset..val_offset + value_bytes_count];
        value_slice.copy_from_slice(val.as_bytes());

        buf_writer.write(&buffer).unwrap();
    }
    buf_writer.flush().unwrap();
}

fn read_index_with_record() {
    let file = std::fs::File::open("index_data.bin").expect("couldn't open file");
    let mut buf_reader = BufReader::new(file);
    const RECORD_SIZE: usize = 4 + 6 + 8;
    let mut buffer = [b'0'; 4 * 1024 * RECORD_SIZE]; //   i<key> + name<value> + header (fixed for now)
    loop {
        let bytes = buf_reader.read(&mut buffer).unwrap();
        if bytes == 0 {
            println!("finished");
            break;
        }
        println!("read bytes {}", bytes);
        let mut consumed_bytes = 0;
        while consumed_bytes + RECORD_SIZE <= bytes {
            let header = &buffer[consumed_bytes..consumed_bytes + 8];
            let key = &buffer[&consumed_bytes + 8..consumed_bytes + 8 + 4];
            let value = &buffer[&consumed_bytes + 8 + 4..consumed_bytes + 8 + 4 + 6];

            let key = u32::from_ne_bytes(key.try_into().unwrap());

            consumed_bytes += RECORD_SIZE;
        }
    }
}

pub fn main() {
    // let args :Vec<String>= env::args().collect();
    // if args.len() != 3 {
    //     eprintln!("invalid args <write|read> <same|seperate>");
    //     return;
    // }
    // let op_mode = &args[1];
    // let op_type = &args[2];
    // let operation = match op_mode {
    //     "write" => {

    //     }
    //     "read" => {

    //     }
    //     _ => {

    //     }
    // }
    // write_index_with_record();
    read_index_with_record();
}
