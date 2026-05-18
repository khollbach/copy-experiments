use std::{
    hint,
    io::{self, Write},
    time::Instant,
};

use anyhow::Result;

/// 4 Gi
const BIG_NUMBER: usize = 4 * 1024 * 1024 * 1024;

fn main() -> Result<()> {
    println!("big number: {}", BIG_NUMBER);

    print!("busy loop ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    for i in 0..BIG_NUMBER {
        hint::black_box(i);
    }
    println!("{:?}", start.elapsed());

    let buf = hint::black_box(vec![0u32; BIG_NUMBER / 4]);
    print!("read (4 bytes at a time) ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    for b in buf {
        hint::black_box(b);
    }
    println!("{:?}", start.elapsed());

    let buf = hint::black_box(vec![0u8; BIG_NUMBER]);
    print!("read (1 byte at a time) ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    for b in buf {
        hint::black_box(b);
    }
    println!("{:?}", start.elapsed());

    let mut buf = vec![0u32; BIG_NUMBER / 4];
    print!("write (4 bytes at a time) ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    buf.fill(0x1234_5678);
    println!("{:?}", start.elapsed());
    hint::black_box(buf);

    let mut buf = vec![0u8; BIG_NUMBER];
    print!("write (1 byte at a time) ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    buf.fill(0x55);
    println!("{:?}", start.elapsed());
    hint::black_box(buf);

    let mut buf = vec![0u32; BIG_NUMBER / 4];
    print!("write zero (4 bytes at a time) ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    buf.fill(0);
    println!("{:?}", start.elapsed());
    hint::black_box(buf);

    let mut buf = vec![0u8; BIG_NUMBER];
    print!("write zero (1 byte at a time) ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    buf.fill(0);
    println!("{:?}", start.elapsed());
    hint::black_box(buf);

    let input = hint::black_box(vec![0u32; BIG_NUMBER / 4]);
    let mut output = vec![0u32; BIG_NUMBER / 4];
    print!("copy (4-byte values ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    output.copy_from_slice(&input);
    println!("{:?}", start.elapsed());
    hint::black_box(output);

    let input = hint::black_box(vec![0u8; BIG_NUMBER]);
    let mut output = vec![0u8; BIG_NUMBER];
    print!("copy (1-byte values) ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    output.copy_from_slice(&input);
    println!("{:?}", start.elapsed());
    hint::black_box(output);

    Ok(())
}

/*
    here's the output of an example run:
big number: 4294967296
busy loop ... 982.881398ms
read (4 bytes at a time) ... 603.324031ms
read (1 byte at a time) ... 1.139239625s
write (4 bytes at a time) ... 1.24425033s
write (1 byte at a time) ... 1.852499388s
write zero (4 bytes at a time) ... 1.874602316s
write zero (1 byte at a time) ... 1.872164469s
copy (4-byte values ... 2.401692797s
copy (1-byte values) ... 2.41952545s

    my takeaways:
- doing ~a billion things takes ~about a second (*very* roughly)
- doing things four-bytes-at-a-time is faster than one-byte-at-a-time
- reading is faster than writing (takes about half or a third as long)
- copying (reading and then writing) is (~)about as fast as reading plus writing
    - copying seems like it doesn't benefit from the 4-byte speedup (?)
- Q: I don't understand why writing 0x1234_5678 is faster than writing 0x0000_0000
    - what's going on here?
*/
