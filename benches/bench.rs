use std::{
    cmp::min,
    hint,
    io::{Cursor, Write},
    slice,
};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

criterion_main!(benches);
// criterion_group!(benches, buffer_size, misc, compression);
criterion_group!(benches, compression);

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
    let mut g = c.benchmark_group("text compression");

    let m = 1024 * 1024;
    // for n in [32 * m, 64 * m, 128 * m] {
    for n in [m] {
        // let zeros = vec![0; n];

        // let mut random = vec![0; n];
        // rand::fill(&mut random);

        let mut text = Vec::with_capacity(n);
        while text.len() < n {
            let alice = include_bytes!("../alice29.txt");
            let chunk_len = min(alice.len(), n - text.len());
            text.extend_from_slice(&alice[..chunk_len]);
        }

        // for (input_name, input) in [("zeros", zeros), ("random", random), ("text", text)] {
        for (_input_name, input) in [("text", text)] {
            let mut output = vec![0; n * 2];

            // let id = BenchmarkId::new("memcpy", n);
            // g.bench_function(id, |b| b.iter(|| copy(&input, &mut output[..n])));

            let mut compressed_input = vec![0; n * 2];
            let len = snap::raw::Encoder::new()
                .compress(&input, &mut compressed_input)
                .unwrap();
            compressed_input.truncate(len);

            let id = BenchmarkId::new("compress", n);
            g.bench_function(id, |b| b.iter(|| my_compress(&input, &mut output)));

            // let id = BenchmarkId::new("compress_reference_impl", n);
            // g.bench_function(id, |b| b.iter(|| snap_compress(&input, &mut output)));

            // let id = BenchmarkId::new("decompress", n);
            // g.bench_function(id, |b| {
            //     b.iter(|| my_decompress(&compressed_input, &mut output))
            // });

            // let id = BenchmarkId::new("decompress_reference_impl", n);
            // g.bench_function(id, |b| {
            //     b.iter(|| snap_decompress(&compressed_input, &mut output))
            // });
        }
    }

    g.finish();
}

fn my_compress(input: &[u8], mut output: &mut [u8]) {
    // todo: extra copy -- does it matter?
    let buf = snippy::compress(input);
    output.write_all(&buf).unwrap();
}

fn my_decompress(input: &[u8], mut output: &mut [u8]) {
    let buf = snippy::decompress(input).unwrap();
    output.write_all(&buf).unwrap();
}

fn snap_compress(input: &[u8], output: &mut [u8]) {
    snap::raw::Encoder::new().compress(input, output).unwrap();
}

fn snap_decompress(input: &[u8], output: &mut [u8]) {
    snap::raw::Decoder::new().decompress(input, output).unwrap();
}
