tobiuo

yet another osakana simulator

features

- accpets osakana input
- 1v1 and nvn simulation
- reasonably fast

install

1. get rust nightly (<https://rustup.rs/>)
2. run `cargo build --release`

todo

- [x] implement 1v1 simulator
- [x] implement NvN simulator (based on JSON config)
- [ ] break out into library with cli (& webui?)
- [ ] benchmark + improve performance
- [ ] port implmentation to CUDA?
- [ ] explore genetic algorithms?

json config example

```
{
  'foo.uo': 10,
  'bar.uo': 10
}
```
