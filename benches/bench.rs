use std::{
    cmp::min,
    hint,
    io::{Cursor, Write},
    slice,
};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

criterion_main!(benches);
criterion_group!(benches, buffer_size, misc, compression);

fn buffer_size(c: &mut Criterion) {
    let mut g = c.benchmark_group("buffer size");

    let n = 1024 * 1024 * 1024; // 1 Gi

    // 64 Ki ..
    for buf_len_log in 16.. {
        let buf_len = 1 << buf_len_log;
        if buf_len > n {
            break;
        }

        let mut in_buf = vec![0; n];
        let mut tmp_buf = vec![0; buf_len];
        let mut out_buf = vec![0; n];
        rand::fill(&mut in_buf);

        let id = BenchmarkId::new("copy", buf_len_log);
        g.bench_function(id, |b| b.iter(|| copy(&in_buf, &mut out_buf)));

        let id = BenchmarkId::new("copy_via_buffer", buf_len_log);
        g.bench_function(id, |b| {
            b.iter(|| copy_via_buffer(&in_buf, &mut tmp_buf, &mut out_buf))
        });
    }

    g.finish();
}

fn copy(input: &[u8], output: &mut [u8]) {
    output.copy_from_slice(input);
}

fn copy_via_buffer(input: &[u8], buf: &mut [u8], output: &mut [u8]) {
    assert_eq!(input.len(), output.len());
    let n = input.len();
    assert_eq!(n % buf.len(), 0);

    let mut output = Cursor::new(output);
    for chunk in input.chunks(buf.len()) {
        buf.copy_from_slice(chunk);
        output.write_all(buf).unwrap();
    }
}

fn misc(c: &mut Criterion) {
    let mut g = c.benchmark_group("misc");

    let n = 1024 * 1024 * 1024; // 1 Gi

    let id = BenchmarkId::new("busy_loop", n);
    g.bench_function(id, |b| b.iter(|| busy_loop(n)));

    let mut input = vec![0; n];
    rand::fill(&mut input);

    let id = BenchmarkId::new("read", n);
    g.bench_function(id, |b| b.iter(|| read(&input)));

    let mut output = vec![0; n];

    let id = BenchmarkId::new("write", n);
    g.bench_function(id, |b| b.iter(|| write(&mut output, 0xab)));

    let id = BenchmarkId::new("copy", n);
    g.bench_function(id, |b| b.iter(|| copy(&input, &mut output)));

    g.finish();
}

fn busy_loop(n: usize) {
    for i in 0..n {
        hint::black_box(i); // prevent the compiler from optimizing away the whole loop
    }
}

fn read(input: &[u8]) {
    assert_eq!(input.len() % 4, 0);
    let input = unsafe { slice::from_raw_parts::<u32>(input.as_ptr() as _, input.len() / 4) };

    for word in input {
        hint::black_box(word);
    }
}

fn write(output: &mut [u8], value: u8) {
    output.fill(value);
}

fn compression(c: &mut Criterion) {
    let mut g = c.benchmark_group("compression");

    // let n = 1024 * 1024 * 1024; // 1 Gi
    let n = 128 * 1024 * 1024; // 128 Mi

    let zeros = vec![0; n];

    let mut random = vec![0; n];
    rand::fill(&mut random);

    let mut text = Vec::with_capacity(n);
    while text.len() < n {
        let alice = include_bytes!("../alice29.txt");
        let chunk_len = min(alice.len(), n - text.len());
        text.extend_from_slice(&alice[..chunk_len]);
    }

    // for (input_name, input) in [("text", text)] {
    for (input_name, input) in [("zeros", zeros), ("random", random), ("text", text)] {
        let mut output = vec![0; n * 2];

        // for comparison
        {
            // (only a few microseconds; doesn't come into the picture)
            // let id = BenchmarkId::new("alloc", input_name);
            // g.bench_function(id, |b| b.iter(|| alloc_and_zero(n)));

            let id = BenchmarkId::new("memcpy", input_name);
            g.bench_function(id, |b| b.iter(|| copy(&input, &mut output[..n])));
        }

        {
            let mut compressed_input = vec![0; n * 2];
            let len = snap::raw::Encoder::new()
                .compress(&input, &mut compressed_input)
                .unwrap();
            compressed_input.truncate(len);

            let id = BenchmarkId::new("my_compress", input_name);
            g.bench_function(id, |b| b.iter(|| my_compress(&input, &mut output)));

            let id = BenchmarkId::new("my_decompress", input_name);
            g.bench_function(id, |b| {
                b.iter(|| my_decompress(&compressed_input, &mut output))
            });

            let id = BenchmarkId::new("snap_compress", input_name);
            g.bench_function(id, |b| b.iter(|| snap_compress(&input, &mut output)));

            let id = BenchmarkId::new("snap_decompress", input_name);
            g.bench_function(id, |b| {
                b.iter(|| snap_decompress(&compressed_input, &mut output))
            });
        }

        // (Too slow!)
        // {
        //     let mut compressed_input = vec![0; n * 2];
        //     let len = deflate_compress(&input, &mut compressed_input);
        //     compressed_input.truncate(len);

        //     let id = BenchmarkId::new("deflate_compress", input_name);
        //     g.bench_function(id, |b| b.iter(|| deflate_compress(&input, &mut output)));

        //     let id = BenchmarkId::new("deflate_decompress", input_name);
        //     g.bench_function(id, |b| {
        //         b.iter(|| deflate_decompress(&compressed_input, &mut output))
        //     });
        // }

        {
            let compressed_input = zstd::stream::encode_all(input.as_slice(), 0).unwrap();

            let id = BenchmarkId::new("zstd_compress", input_name);
            g.bench_function(id, |b| b.iter(|| zstd_compress(&input, &mut output)));

            let id = BenchmarkId::new("zstd_decompress", input_name);
            g.bench_function(id, |b| {
                b.iter(|| zstd_decompress(&compressed_input, &mut output))
            });
        }
    }

    g.finish();
}

// fn alloc_and_zero(n: usize) {
//     let buf = vec![0; n];
//     hint::black_box(buf);
// }

fn my_compress(input: &[u8], mut output: &mut [u8]) {
    // todo: extra copy -- does it matter?
    let buf = snippy::compress(input);
    output.write_all(&buf).unwrap();
}

fn my_decompress(input: &[u8], mut output: &mut [u8]) {
    let buf = snippy::decompress(input).unwrap();
    output.write_all(&buf).unwrap();
}

//

fn snap_compress(input: &[u8], output: &mut [u8]) {
    snap::raw::Encoder::new().compress(input, output).unwrap();
}

fn snap_decompress(input: &[u8], output: &mut [u8]) {
    snap::raw::Decoder::new().decompress(input, output).unwrap();
}

//

// fn deflate_compress(input: &[u8], output: &mut [u8]) -> usize {
//     let mut co = flate2::Compress::new(flate2::Compression::fast(), false);
//     let st = co
//         .compress(input, output, flate2::FlushCompress::None)
//         .unwrap();
//     assert_eq!(st, flate2::Status::Ok);
//     assert_eq!(co.total_in(), u64::try_from(input.len()).unwrap());
//     usize::try_from(co.total_out()).unwrap()
// }

// fn deflate_decompress(input: &[u8], output: &mut [u8]) {
//     let mut de = flate2::Decompress::new(false);
//     let st = de
//         .decompress(input, output, flate2::FlushDecompress::None)
//         .unwrap();
//     assert_eq!(st, flate2::Status::Ok);
//     assert_eq!(de.total_in(), u64::try_from(input.len()).unwrap());
// }

//

fn zstd_compress(input: &[u8], mut output: &mut [u8]) {
    let buf = zstd::stream::encode_all(input, 0).unwrap();
    output.write_all(&buf).unwrap();
}

fn zstd_decompress(input: &[u8], mut output: &mut [u8]) {
    let buf = zstd::stream::decode_all(input).unwrap();
    output.write_all(&buf).unwrap();
}
