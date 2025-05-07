use serde::{Deserialize, Serialize};

shadow_rs::shadow!(build);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildInfo {
    pub branch: &'static str,
    pub commit: &'static str,
    pub commit_short: &'static str,
    pub clean: bool,
    pub source_time: &'static str,
    pub build_time: &'static str,
    pub rustc: &'static str,
    pub target: &'static str,
    pub version: &'static str,
}

pub const fn build_info() -> BuildInfo {
    BuildInfo {
        branch: build::BRANCH,
        commit: build::COMMIT_HASH,
        commit_short: build::SHORT_COMMIT,
        clean: build::GIT_CLEAN,
        source_time: env!("SOURCE_TIMESTAMP"),
        build_time: env!("BUILD_TIMESTAMP"),
        rustc: build::RUST_VERSION,
        target: build::BUILD_TARGET,
        version: build::PKG_VERSION,
    }
}

pub const fn version() -> &'static str {
    const BUILD_INFO: BuildInfo = build_info();

    const_format::formatcp!(
        "\nversion: {}\nbranch: {}\ncommit: {}\nclean: {}\nsource_time: {}\nbuild_time: {}\nrustc: {}\ntarget: {}",
        BUILD_INFO.version,
        BUILD_INFO.branch,
        BUILD_INFO.commit,
        BUILD_INFO.clean,
        BUILD_INFO.source_time,
        BUILD_INFO.build_time,
        BUILD_INFO.rustc,
        BUILD_INFO.target,
    )
}
