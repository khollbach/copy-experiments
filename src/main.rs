use std::{
    cmp::min,
    hint,
    io::{self, Write},
    time::Instant,
};

use anyhow::Result;
use snap::raw::{self, Decoder, Encoder};

fn _main() -> Result<()> {
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

fn __main() -> Result<()> {
    println!("big number: {}", BIG_NUMBER);

    print!("busy loop ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    for i in 0..BIG_NUMBER {
        hint::black_box(i);
    }
    println!("{:?}", start.elapsed());

    let input = hint::black_box(vec![0u8; BIG_NUMBER]);
    let mut output = vec![0u8; BIG_NUMBER];
    print!("copy directly ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    output.copy_from_slice(&input);
    println!("{:?}", start.elapsed());
    hint::black_box(output);

    let input = hint::black_box(vec![0u8; BIG_NUMBER]);
    let mut output = Vec::with_capacity(BIG_NUMBER);
    let buf_size = 64 * 1024;
    let mut buf = vec![0u8; buf_size];
    print!("copy via 64K buffer ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    for chunk in input.chunks(buf_size) {
        buf.copy_from_slice(chunk);
        output.extend_from_slice(&buf);
    }
    println!("{:?}", start.elapsed());
    hint::black_box(output);

    Ok(())
}

/*
    output:
big number: 4294967296
busy loop ... 520.625526ms
copy directly ... 2.438076909s
copy via 64K buffer ... 2.175316407s

- incredibly, copying via the buffer seems to actually speed things up!
- I guess the buffer fits into the CPU's cache (?), and the access is so fast
  that it can be ignored, compared to the reads/writes of the huge input/output
  arrays
- so I guess my concern that we'll be introducing "an extra copy" if we write
  the compression code a certain way isn't a big deal after all.
*/

/// 1 Gi
const BIG_NUMBER: usize = 1 * 1024 * 1024 * 1024;

fn main() -> Result<()> {
    println!("big number: {}", BIG_NUMBER);

    print!("busy loop ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    for i in 0..BIG_NUMBER {
        hint::black_box(i);
    }
    println!("{:?}", start.elapsed());

    let input = hint::black_box(vec![0u8; BIG_NUMBER]);
    let mut output = vec![0u8; BIG_NUMBER];
    print!("memcopy ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    output.copy_from_slice(&input);
    println!("{:?}", start.elapsed());
    hint::black_box(output);

    let alice = include_bytes!("../alice29.txt");
    let mut input = Vec::with_capacity(BIG_NUMBER);
    while input.len() < BIG_NUMBER {
        let chunk_len = min(alice.len(), BIG_NUMBER - input.len());
        input.extend_from_slice(&alice[..chunk_len]);
    }

    // (Compile error; ignoring for now.)
    // // let mut compressed = Vec::with_capacity(BIG_NUMBER);
    // print!("compress (c snappy) ... ");
    // io::stdout().flush()?;
    // let start = Instant::now();
    // let compressed = snappy::compress(&input);
    // println!("{:?}", start.elapsed());
    // hint::black_box(compressed);
    // // let mut decompressed = Vec::with_capacity(BIG_NUMBER);
    // print!("decompress (c snappy) ... ");
    // io::stdout().flush()?;
    // let start = Instant::now();
    // let decompressed = snappy::decompress(&compressed)?;
    // println!("{:?}", start.elapsed());
    // hint::black_box(decompressed);

    print!("compress (rust-snappy) ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    let mut compressed = vec![0; raw::max_compress_len(input.len())];
    let len = compress(&input, &mut compressed)?;
    compressed.truncate(len);
    println!("{:?}", start.elapsed());
    hint::black_box(&compressed);

    print!("decompress (rust-snappy) ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    let mut decompressed = vec![0; input.len()];
    let len = decompress(&compressed, &mut decompressed)?;
    decompressed.truncate(len);
    println!("{:?}", start.elapsed());
    hint::black_box(decompressed);

    println!("(compressed len: {})", compressed.len()); // around half

    print!("compress (my impl) ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    let _compressed = snippy::compress(&input);
    println!("{:?}", start.elapsed());
    hint::black_box(&_compressed);

    let mut decompressed = Vec::with_capacity(BIG_NUMBER);
    decompressed.resize(BIG_NUMBER, 0);
    print!("decompress (my impl) ... ");
    io::stdout().flush()?;
    let start = Instant::now();
    let decompressed = snippy::decompress(&compressed)?;
    println!("{:?}", start.elapsed());
    hint::black_box(decompressed);

    Ok(())
}

fn compress(input: &[u8], output: &mut [u8]) -> Result<usize> {
    Ok(Encoder::new().compress(input, output)?)
}

fn decompress(input: &[u8], output: &mut [u8]) -> Result<usize> {
    Ok(Decoder::new().decompress(input, output)?)
}

/*
big number: 1073741824
busy loop ... 144.225547ms
memcopy ... 661.194116ms
compress (rust-snappy) ... 2.511677024s
decompress (rust-snappy) ... 1.131712414s
(compressed len: 616777753)
compress (my impl) ... 5.309834987s
decompress (my impl) ... 2.66605313s

Ok cool. So my compress impl takes twice as long -- that's fair enough.
- There's lots we could probably optimize, and I'll declare it "out of scope" for now...

But why does my decompress impl take more than twice as long?
- I feel like the code is pretty straightforward, so I'm curious what's causing
  such a big difference!

Also cool to note that decompress takes only about twice as long as a memcpy.
(And compress about twice as long as that.)
*/
