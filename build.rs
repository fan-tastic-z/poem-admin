use std::{collections::BTreeSet, env, path::Path};

use build_data::{format_timestamp, get_source_time};
use shadow_rs::{CARGO_METADATA, CARGO_TREE, ShadowBuilder};

fn configure_rerun_if_head_commit_changed() {
    let mut current = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();

    // skip if no valid-looking git repository could be found
    while let Ok((dir, _)) = gix_discover::upwards(current.as_path()) {
        match dir {
            gix_discover::repository::Path::Repository(git_dir) => {
                unreachable!(
                    "build.rs should never be placed in a git bare repository: {}",
                    git_dir.display()
                );
            }
            gix_discover::repository::Path::WorkTree(work_dir) => {
                let git_refs_heads = work_dir.join(".git/refs/heads");
                println!("cargo::rerun-if-changed={}", git_refs_heads.display());
                break;
            }
            gix_discover::repository::Path::LinkedWorkTree { work_dir, .. } => {
                current = work_dir
                    .parent()
                    .expect("submodule's work_dir must have parent")
                    .to_path_buf();
                continue;
            }
        };
    }
}

fn main() -> shadow_rs::SdResult<()> {
    configure_rerun_if_head_commit_changed();

    println!(
        "cargo::rustc-env=SOURCE_TIMESTAMP={}",
        if let Ok(t) = get_source_time() {
            format_timestamp(t).unwrap_or_default()
        } else {
            String::new()
        }
    );
    build_data::set_BUILD_TIMESTAMP();

    // The "CARGO_WORKSPACE_DIR" is set manually (not by Rust itself) in Cargo config file, to
    // solve the problem where the "CARGO_MANIFEST_DIR" is not what we want when this repo is
    // made as a submodule in another repo.
    let src_path = env::var("CARGO_WORKSPACE_DIR").or_else(|_| env::var("CARGO_MANIFEST_DIR"))?;
    let out_path = env::var("OUT_DIR")?;
    let _ = ShadowBuilder::builder()
        .src_path(src_path)
        .out_path(out_path)
        // exclude these two large constants that we don't need
        .deny_const(BTreeSet::from([CARGO_METADATA, CARGO_TREE]))
        .build()?;
    Ok(())
}
