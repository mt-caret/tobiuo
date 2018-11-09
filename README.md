tobiuo

yet another osakana simulator

features

- accpets osakana input
- 1v1 and nvn simulation
- reasonably fast

build

1. get rust nightly (<https://rustup.rs/>)
2. run `cargo build --release`

todo

- [x] implement 1v1 simulator
- [x] implement NvN simulator (based on JSON config)
- [ ] break out into library with cli (& webui?)
- [x] benchmark + improve performance
- [ ] port implmentation to CUDA?
- [ ] explore genetic algorithms?

json config example

```
{
  'foo.uo': 10,
  'bar.uo': 10
}
```

performance

```
test tests::bench_simulate ... bench:     371,554 ns/iter (+/- 16,269)
```

`tests::bench_simulate` runs `simulate()` 1000 times, so this adds up to
simulating close to 2.7 million games a second.
