use std::{
    cell::RefCell,
    hint,
    io::{Cursor, Write},
};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

criterion_main!(benches);
criterion_group!(benches, bench);

fn bench(c: &mut Criterion) {
    let mut g = c.benchmark_group("copy");

    let n = 1024 * 1024 * 1024; // 1 Gi
    // let n = 1024 * 1024; // 1 Mi
    // let n = 512 * 1024 * 1024; // 512 Mi

    // 64 Ki ..
    for buf_len_log in 16..18 {
        let buf_len = 1 << buf_len_log;
        if buf_len > n {
            break;
        }

        let mut input = Input {
            in_buf: vec![0; n],
            tmp_buf: vec![0; buf_len].into(),
            out_buf: vec![0; n].into(),
        };
        rand::fill(&mut input.in_buf);

        let id = BenchmarkId::new("copy", buf_len_log);
        g.bench_with_input(id, &input, |b, input| {
            b.iter(|| copy(&input.in_buf, &mut input.out_buf.borrow_mut()))
        });

        let id = BenchmarkId::new("copy_via_buffer", buf_len_log);
        g.bench_with_input(id, &input, |b, input| {
            b.iter(|| {
                copy_via_buffer(
                    &input.in_buf,
                    &mut input.tmp_buf.borrow_mut(),
                    &mut input.out_buf.borrow_mut(),
                )
            })
        });
    }

    g.finish();
}

struct Input {
    in_buf: Vec<u8>,
    tmp_buf: RefCell<Vec<u8>>,
    out_buf: RefCell<Vec<u8>>,
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
