# reasonable_envsubst
![crates.io](https://img.shields.io/crates/v/reasonable_envsubst?label=latest)
![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.74+-ab6000.svg)
![github](https://github.com/${GITHUB_NAME}/reasonable_envsubst)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/${GITHUB_NAME}/reasonable_envsubst/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/${GITHUB_NAME}/reasonable_envsubst/actions?query=branch%3Amaster)
# BUG
Panics when there is ${env_var} that is not found locally.

Desired behavior:
Skip over. Github Actions often reference vars defined within them or on github remote.

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
