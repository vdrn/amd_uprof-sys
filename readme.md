# amd_uprof-sys

Provides FFI bindings to AMD uProf's `AMDProfileController` API for profiling from Rust.

## Installation
```toml 
[dependencies]
amd_uprof-sys = "0.1"
```


## Env vars
- `AMD_UPROF_DIR`: root install directory; used to infer include/lib paths.
- `AMD_UPROF_INCLUDE_DIR`: explicit path to headers (overrides inference).
- `AMD_UPROF_LIB_DIR`: explicit path to libraries (overrides inference).

If env vars are not set, the build will try to find AMD uProf headers and libraries in common install locations (e.g. `/opt` or `/usr/local`).

## Features
- `bindgen`: generate bindings at build time from system headers. Without it, the crate uses committed bindings (currently targeting `AMD uProf 5.0`).

