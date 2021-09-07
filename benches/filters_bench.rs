use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::prelude::*;
use smallvec::SmallVec;

pub enum Filter {
    DataSize(u64),
    Memcmp {
        offset: usize,
        bytes: SmallVec<[u8; 128]>,
    },
}

impl Filter {
    pub(crate) fn matches(&self, data: &[u8]) -> bool {
        match self {
            Filter::DataSize(len) => data.len() as u64 == *len,
            Filter::Memcmp { offset, bytes } => {
                let len = bytes.len();
                match data.get(*offset..*offset + len) {
                    Some(slice) => slice == &bytes[..len],
                    None => false,
                }
            }
        }
    }
}

fn test_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen()).collect()
}

/*
fn test_filter(limit: usize) -> Filter {
    let mut rng = rand::thread_rng();
    if rng.gen() {
        Filter::DataSize(rng.gen())
    } else {
        Filter::Memcmp {
            offset: rng.gen_range(0..limit),
            bytes: (0..rng.gen_range(0..limit)).map(|_| rng.gen()).collect(),
        }
    }
}
*/

fn test_filter_group(limit: usize) -> SmallVec<[Filter; 2]> {
    //SmallVec::from([test_filter(limit), test_filter(limit)])
    let mut rng = rand::thread_rng();
    let f1 = Filter::Memcmp {
        offset: rng.gen_range(0..limit),
        bytes: (0..rng.gen_range(0..limit)).map(|_| rng.gen()).collect(),
    };
    let f2 = Filter::DataSize(rng.gen());
    SmallVec::from([f2, f1])
}

fn filter_table(data_limit: usize, count: usize) -> Vec<SmallVec<[Filter; 2]>> {
    (0..count).map(|_| test_filter_group(data_limit)).collect()
}

fn bench_filters(c: &mut Criterion) {
    let mut group = c.benchmark_group("Filters");
    let data = test_data(1024);
    let filter_table = filter_table(1024, 10_000);

    for i in [(data, filter_table)].iter() {
        group.bench_with_input(BenchmarkId::new("Dumb", 1), i, |b, (data, filters)| {
            b.iter(|| dumb(data, filters))
        });
        group.bench_with_input(BenchmarkId::new("NotSoDumb", 1), i, |b, (data, filters)| {
            b.iter(|| new(data, filters))
        });
    }
    group.finish();
}

fn dumb(data: &[u8], table: &[SmallVec<[Filter; 2]>]) -> usize {
    let mut matches = 0;
    for group in table {
        if group.iter().all(|f| f.matches(data)) {
            matches += 1;
        }
    }
    matches
}

fn new(data: &[u8], table: &[SmallVec<[Filter; 2]>]) -> usize {
    let mut matches = 0;
    for group in table {
        if group.iter().all(|f| f.matches(data)) {
            matches += 1;
        }
    }
    matches
}

criterion_group!(benches, bench_filters);
criterion_main!(benches);
