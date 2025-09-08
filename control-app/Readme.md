## Getting started with development

For cross-compilation ensure that you ran the following commands once:

```bash
cargo install cross
rustup target add aarch64-unknown-linux-gnu
```

Then build with:

```
cross build --release --target aarch64-unknown-linux-gnu
```
