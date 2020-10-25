// FileHandle { fd, size, offset }
use std::fs::{File};
use std::io::{Read, Seek, SeekFrom};
use nom::{
    IResult,
    bytes::complete::{tag, take},
    number::complete::{le_u8, le_u32},
    combinator::map_res,
    sequence::tuple
};

const MAGIC: &'static [u8] =  &[0x57, 0xFB, 0x80, 0x8B, 0x24, 0x75, 0x47, 0xDB];
const MAX_ENCODE_LENGTH: usize = 20;
const FOOTER_ENCODE_LENGTH: usize = 2 * MAX_ENCODE_LENGTH + 8;
const BLOCK_TRAILER_SIZE: usize = 5; // 1 type + 4 crc
const COMPRESSION_NO: u8 = 0;
const COMPRESSION_SNAPPY: u8 = 1;

#[derive(Debug)]
struct Handle {
    offset: usize,
    size: usize,
}

#[derive(Debug)]
struct Footer {
    meta_index: Handle,
    index: Handle,
}

fn decode_var_u64(mut input: &[u8]) -> IResult<&[u8], u64> {
    let mut res: u64 = 0;
    for shift in (0..=63).step_by(7) {
        let (input2, b) = le_u8(input)?;
        input = input2;
        if b & 128 != 0 {
            res |= (b as u64 & 127) << shift;
        } else {
            res |= (b as u64) << shift;
            return Ok((input, res));
        }
    }
    Ok((input, res))
}

fn decode_var_u32(mut input: &[u8]) -> IResult<&[u8], u32> {
    let mut res: u32 = 0;
    for shift in (0..=31).step_by(7) {
        let (input2, b) = le_u8(input)?;
        input = input2;
        if b & 128 != 0 {
            res |= (b as u32 & 127) << shift;
        } else {
            res |= (b as u32) << shift;
            return Ok((input, res));
        }
    }
    Ok((input, res))
}

#[derive(Debug)]
struct Block {
    data: Vec<u8>,
    restarts: Vec<usize>,
}

impl Handle {
    fn decode(input: &[u8]) -> IResult<&[u8], Handle> {
        let (input, offset) = decode_var_u64(input)?;
        let (input, size) = decode_var_u64(input)?;
        Ok((input, Handle { offset: offset as usize, size: size as usize }))
    }

    fn read_data(&self, file: &mut File) -> Vec<u8> {
        let mut res =vec![0; self.size];
        file.read_exact(res.as_mut_slice()).unwrap();
        res
    }

    fn read_block(&self, file: &mut File) -> Block {
        let mut res =vec![0; self.size + BLOCK_TRAILER_SIZE];
        file.seek(SeekFrom::Start(self.offset as u64));
        file.read_exact(res.as_mut_slice()).unwrap();
        let compression = res[self.size];
        if compression != COMPRESSION_NO {
            panic!("not supported compression (for now): {}", compression);
        }
        println!("read block: Handle = {:?}, type = {}, crc = {:?}", self, compression, &res[self.size+1..]);
        res.truncate(self.size);
        res.into()
    }
}

impl Footer {
    fn decode(input: &[u8]) -> IResult<&[u8], Footer> {
        // check magic
        tag(MAGIC)(&input[FOOTER_ENCODE_LENGTH-8..])?;
        let (input, meta_index) = Handle::decode(input)?;
        let (input, index) = Handle::decode(input)?;
        Ok((input, Footer { meta_index, index }))
    }

    fn read_index(&self, file: &mut File) -> Vec<Handle> {
        let res = vec![];
        
        res
    }
}

impl From<Vec<u8>> for Block {
    fn from(mut data: Vec<u8>) -> Self {
        let (_, num_restarts) = le_u32::<(&[u8], nom::error::ErrorKind)>(&data[data.len()-4..]).unwrap();
        let num_restarts = num_restarts as usize;
        println!("num_restarts = {}", num_restarts);
        let restarts_offset = data.len() - (4 + num_restarts * 4);
        let mut restarts = Vec::with_capacity(num_restarts);
        for i in 0..num_restarts {
            let (_, offset) = le_u32::<(&[u8], nom::error::ErrorKind)>(&data[restarts_offset+i*4..]).unwrap();
            restarts.push(offset as usize);
        }
        data.truncate(restarts_offset);
        println!("data.len() = {}", data.len());
        Block {
            data,
            restarts,
        }
    }
}

struct BlockIterator<'a> {
    data: &'a [u8],
    key: Vec<u8>,
}

#[derive(Debug)]
struct Entry {
    shared: usize,
    key: Vec<u8>,
    value: Vec<u8>,
}

impl<'a> Iterator for BlockIterator<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let input = self.data;
        if input.is_empty() { return None; }
        let (input, len_shared) = decode_var_u32(input).unwrap();
        let (input, len_non_shared) = decode_var_u32(input).unwrap();
        let (input, len_value) = decode_var_u32(input).unwrap();
        self.key.truncate(len_shared as usize);
        let (input, key) = take::<_, _, (&[u8], nom::error::ErrorKind)>(len_non_shared)(input).unwrap();
        self.key.extend(key);
        let (input, value) = take::<_, _, (&[u8], nom::error::ErrorKind)>(len_value)(input).unwrap();
        self.data = input;
        Some(Entry {
            shared: len_shared as usize,
            key: self.key.clone(),
            value: value.iter().cloned().collect(),
        })
    }
}

impl<'a> IntoIterator for &'a Block {
    type Item = Entry;

    type IntoIter = BlockIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BlockIterator {
            data: &self.data,
            key: vec![],
        }
    }
}

pub fn dumpSSTable(file: &str) {
    let mut f = File::open(file).unwrap();
    f.seek(SeekFrom::End(-(FOOTER_ENCODE_LENGTH as i64)));
    let mut buf: [u8;FOOTER_ENCODE_LENGTH] = [0;FOOTER_ENCODE_LENGTH];
    f.read_exact(&mut buf).unwrap();
    let footer = Footer::decode(&buf).unwrap().1;
    println!("{:?}", footer);
    let blk = footer.index.read_block(&mut f);
    let meta_blk = footer.meta_index.read_block(&mut f);
    // println!("Index block: {:?}", &blk.restarts[..10]);
    // println!("MetaIndex block: {:?}", meta_blk);
    let mut handle = None;
    for (i, entry) in blk.into_iter().enumerate() {
        println!("entry shared {:?}", entry.shared);
        println!("entry key: {:?}", String::from_utf8(entry.key[..(entry.key.len()-8)].to_vec()).unwrap());
        handle = Some(Handle::decode(&entry.value[..]).unwrap().1);
        println!("entry Value as Handle: {:?}", handle.as_ref().unwrap());
        // if i == 10 {
        //     break;
        // }
    }
    handle = Some(Handle { offset: 78122425, size: 4139 });
    let data_blk = handle.unwrap().read_block(&mut f);
    for entry in data_blk.into_iter() {
        println!("entry shared = {}", entry.shared);
        println!("entry key = {:?}", String::from_utf8(entry.key[..(entry.key.len()-8)].to_vec()).unwrap());
        println!("entry value = {:?}", String::from_utf8(entry.value).unwrap());
    }
}