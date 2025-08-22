use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if env::var("DOCS_RS").is_ok() {
        return;
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-env-changed=AMD_UPROF_DIR");
    println!("cargo:rerun-if-env-changed=AMD_UPROF_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=AMD_UPROF_LIB_DIR");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| String::from("linux"));

    let lib_dir = find_lib_dir(&target_os, None).expect(
        "Could not locate AMD uProf libraries. Set AMD_UPROF_LIB_DIR or AMD_UPROF_DIR to the install path.",
    );
    println!("cargo:rustc-link-search={}", lib_dir.display());
    println!("cargo:rustc-link-lib=AMDProfileController");

    #[cfg(feature = "bindgen")]
    {
        const INCLUDE_SUBDIRS: &[&str] = &["include", "inc"];
        let include_dir = find_with_env_and_roots(
            "AMD_UPROF_INCLUDE_DIR",
            "AMD_UPROF_DIR",
            INCLUDE_SUBDIRS,
            |dir| dir.join("AMDProfileController.h").exists()
        )
        .expect(
            "Could not locate AMD uProf headers. Set AMD_UPROF_INCLUDE_DIR or AMD_UPROF_DIR to the install path",
        );

        let mut builder = bindgen::Builder::default()
            .header("wrapper.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .clang_arg(format!("-I{}", include_dir.display()));

        let bindings = builder
            .generate()
            .expect("Unable to generate bindings. Ensure AMD uProf headers are installed or set AMD_UPROF_INCLUDE_DIR/AMD_UPROF_DIR");

        let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Could not write bindings!");
    }
}

fn find_lib_dir(target_os: &str, include_dir: Option<&Path>) -> Option<PathBuf> {
    const LIB_SUBDIRS: &[&str] = &["lib", "lib64", "lib/x64"];

    let found = find_with_env_and_roots("AMD_UPROF_LIB_DIR", "AMD_UPROF_DIR", LIB_SUBDIRS, |dir| {
        lib_exists_in_dir(target_os, dir)
    });
    if found.is_some() {
        return found;
    }

    // If we found an include dir, try siblings (in case of no override)
    if let Some(inc) = include_dir
        && let Some(root) = inc.parent()
    {
        for candidate in collect_candidates_from_root(root, LIB_SUBDIRS) {
            if lib_exists_in_dir(target_os, &candidate) {
                return Some(candidate);
            }
        }
    }

    None
}

fn lib_exists_in_dir(target_os: &str, dir: &Path) -> bool {
    match target_os {
        "windows" => {
            dir.join("AMDProfileController.lib").exists()
                || dir.join("AMDProfileController.dll").exists()
        }
        "macos" => dir.join("libAMDProfileController.dylib").exists(),
        _ => {
            dir.join("libAMDProfileController.so").exists()
                || dir.join("libAMDProfileController.a").exists()
        }
    }
}

fn collect_candidates_from_root(root: &Path, subdirs: &[&str]) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for sub in subdirs {
        candidates.push(root.join(sub));
    }
    // scan one level down in case of nested installs
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                for sub in subdirs {
                    candidates.push(path.join(sub));
                }
            }
        }
    }
    candidates
}

fn common_roots() -> Vec<PathBuf> {
    let base_roots = [PathBuf::from("/opt"), PathBuf::from("/usr/local")];
    let mut expanded = Vec::new();
    for root in base_roots {
        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("amduprof") || name.contains("amd_uprof") {
                    expanded.push(path);
                }
            }
        }
    }
    expanded
}

fn find_with_env_and_roots(
    dir_env: &str,
    root_env: &str,
    subdirs: &[&str],
    mut exists: impl FnMut(&Path) -> bool,
) -> Option<PathBuf> {
    // dir override
    if let Ok(dir) = env::var(dir_env) {
        let path = PathBuf::from(&dir);
        if exists(&path) {
            return Some(path);
        }
    }

    // root override
    if let Ok(root) = env::var(root_env) {
        let root_path = PathBuf::from(&root);
        for candidate in collect_candidates_from_root(&root_path, subdirs) {
            if exists(&candidate) {
                return Some(candidate);
            }
        }
    }

    // search in common locations
    for root in &common_roots() {
        for candidate in collect_candidates_from_root(root, subdirs) {
            if exists(&candidate) {
                return Some(candidate);
            }
        }
    }

    None
}
