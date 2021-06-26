use criterion::{black_box, criterion_group, criterion_main, Criterion};

use treebender::Grammar;

const GRAMMAR_SRC: &str = include_str!("./reflexives.fgr");

fn parse(g: &Grammar, input: &[&str]) -> usize {
  g.parse(input).len()
}

fn criterion_benchmark(c: &mut Criterion) {
  let grammar = GRAMMAR_SRC.parse::<Grammar>().unwrap();
  let simple_input = "mary likes sue".split(' ').collect::<Vec<_>>();
  let complex_input = "mary said that she likes herself"
    .split(' ')
    .collect::<Vec<_>>();

  c.bench_function("parse simple", |b| {
    b.iter(|| parse(black_box(&grammar), black_box(&simple_input)))
  });

  c.bench_function("parse complex reflexive", |b| {
    b.iter(|| parse(black_box(&grammar), black_box(&complex_input)))
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
