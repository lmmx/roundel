Run `cargo fmt` on your code and then build with `cargo build` and then if OK build for release with

```bash
trunk build --release
```

The `dist/` directory will then have the files you need to copy to deploy on a static site host.
