//! Helper script to publish the warg suites of crates
//!
//! * `./publish bump` - bump crate versions in-tree
//! * `./publish verify` - verify crates can be published to crates.io
//! * `./publish publish` - actually publish crates to crates.io

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use clap::Parser;

// note that this list must be topologically sorted by dependencies
const SORTED_CRATES_TO_PUBLISH: &[&str] = &[
    "wdl-grammar",
    "wdl-ast",
    "wdl-lint",
    "wdl-analysis",
    "wdl-lsp",
    "wdl",
];

struct Crate {
    manifest: cargo_toml::Manifest,
    path: PathBuf,
    name: String,
    version: String,
    publish: bool,
}

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Bump(Bump),
    Publish(Publish),
    Verify(Verify),
}

#[derive(Parser)]
struct Bump {
    #[clap(short, long)]
    patch: bool,
}

#[derive(Parser)]
struct Publish {}

#[derive(Parser)]
struct Verify {}

fn main() {
    let mut crates = Vec::new();
    find_crates(".".as_ref(), &mut crates);

    let publish_order = SORTED_CRATES_TO_PUBLISH
        .iter()
        .enumerate()
        .map(|(i, c)| (*c, i))
        .collect::<HashMap<_, _>>();
    crates.sort_by_key(|krate| publish_order.get(&krate.name[..]));

    let opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Bump(Bump { patch }) => {
            for krate in crates.iter() {
                bump_version(krate, &crates, patch);
            }
            // update the lock file
            assert!(
                Command::new("cargo")
                    .arg("fetch")
                    .status()
                    .unwrap()
                    .success()
            );
        }
        SubCommand::Publish(_) => {
            // We have so many crates to publish we're frequently either
            // rate-limited or we run into issues where crates can't publish
            // successfully because they're waiting on the index entries of
            // previously-published crates to propagate. This means we try to
            // publish in a loop and we remove crates once they're successfully
            // published. Failed-to-publish crates get enqueued for another try
            // later on.
            for _ in 0..10 {
                crates.retain(|krate| !publish(krate));

                if crates.is_empty() {
                    break;
                }

                println!(
                    "{} crates failed to publish, waiting for a bit to retry",
                    crates.len(),
                );
                thread::sleep(Duration::from_secs(40));
            }

            assert!(crates.is_empty(), "failed to publish all crates");

            println!();
            println!("===================================================================");
            println!();
            println!("Don't forget to push a git tag for this release!");
            println!("    $ git push git@github.com:bytecodealliance/cargo-component.git vX.Y.Z");
        }
        SubCommand::Verify(_) => {
            verify(&crates);
        }
    }
}

fn find_crates(dir: &Path, dst: &mut Vec<Crate>) {
    if dir.join("Cargo.toml").exists() {
        if let Some(krate) = read_crate(&dir.join("Cargo.toml")) {
            dst.push(krate);
        }
    }

    for entry in dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            find_crates(&entry.path(), dst);
        }
    }
}

fn read_crate(manifest_path: &Path) -> Option<Crate> {
    let manifest =
        cargo_toml::Manifest::from_path(manifest_path).expect("failed to parse manifest");
    let package = match manifest.package {
        Some(ref p) => p,
        None => return None, // workspace manifests don't have a package section
    };
    let name = package.name().to_string();
    let version = package.version().to_string();
    let publish = match package.publish() {
        cargo_toml::Publish::Flag(b) => b.to_owned(),
        _ => true,
    };
    Some(Crate {
        manifest,
        path: manifest_path.to_path_buf(),
        name,
        version,
        publish,
    })
}

fn bump_version(krate: &Crate, crates: &[Crate], patch: bool) {
    let next_version = |krate: &Crate| -> String {
        if SORTED_CRATES_TO_PUBLISH.contains(&&krate.name[..]) {
            bump(&krate.version, patch)
        } else {
            krate.version.clone()
        }
    };

    let mut new_manifest = krate.manifest.clone();
    new_manifest
        .package
        .as_mut()
        .expect("should be a package")
        .version = cargo_toml::Inheritable::Set(next_version(krate));

    // TODO: update dependencies

    fs::write(
        &krate.path,
        toml::to_string(&new_manifest).expect("failed to serialize new manifest"),
    )
    .expect("failed to write new manifest");
}

/// Performs a major version bump increment on the semver version `version`.
///
/// This function will perform a semver-major-version bump on the `version`
/// specified. This is used to calculate the next version of a crate in this
/// repository since we're currently making major version bumps for all our
/// releases. This may end up getting tweaked as we stabilize crates and start
/// doing more minor/patch releases, but for now this should do the trick.
fn bump(version: &str, patch_bump: bool) -> String {
    let mut iter = version.split('.').map(|s| s.parse::<u32>().unwrap());
    let major = iter.next().expect("major version");
    let minor = iter.next().expect("minor version");
    let patch = iter.next().expect("patch version");

    if patch_bump {
        return format!("{}.{}.{}", major, minor, patch + 1);
    }
    if major != 0 {
        format!("{}.0.0", major + 1)
    } else if minor != 0 {
        format!("0.{}.0", minor + 1)
    } else {
        format!("0.0.{}", patch + 1)
    }
}

fn publish(krate: &Crate) -> bool {
    if !SORTED_CRATES_TO_PUBLISH.iter().any(|s| *s == krate.name) {
        return true;
    }

    // First make sure the crate isn't already published at this version. This
    // script may be re-run and there's no need to re-attempt previous work.
    let output = Command::new("curl")
        .arg(&format!("https://crates.io/api/v1/crates/{}", krate.name))
        .output()
        .expect("failed to invoke `curl`");
    if output.status.success()
        && String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"newest_version\":\"{}\"", krate.version))
    {
        println!(
            "skip publish {} because {} is latest version",
            krate.name, krate.version,
        );
        return true;
    }

    let status = Command::new("cargo")
        .arg("publish")
        .current_dir(krate.path.parent().unwrap())
        .arg("--no-verify")
        .status()
        .expect("failed to run cargo");
    if !status.success() {
        println!("FAIL: failed to publish `{}`: {}", krate.name, status);
        return false;
    }

    // After we've published then make sure that the `wasmtime-publish` group is
    // added to this crate for future publications. If it's already present
    // though we can skip the `cargo owner` modification.
    let output = Command::new("curl")
        .arg(&format!(
            "https://crates.io/api/v1/crates/{}/owners",
            krate.name
        ))
        .output()
        .expect("failed to invoke `curl`");
    if output.status.success()
        && String::from_utf8_lossy(&output.stdout).contains("wasmtime-publish")
    {
        println!(
            "wasmtime-publish already listed as an owner of {}",
            krate.name
        );
        return true;
    }

    // Note that the status is ignored here. This fails most of the time because
    // the owner is already set and present, so we only want to add this to
    // crates which haven't previously been published.
    let status = Command::new("cargo")
        .arg("owner")
        .arg("-a")
        .arg("github:bytecodealliance:wasmtime-publish")
        .arg(&krate.name)
        .status()
        .expect("failed to run cargo");
    if !status.success() {
        panic!(
            "FAIL: failed to add wasmtime-publish as owner `{}`: {}",
            krate.name, status
        );
    }

    true
}

// Verify the current tree is publish-able to crates.io. The intention here is
// that we'll run `cargo package` on everything which verifies the build as-if
// it were published to crates.io. This requires using an incrementally-built
// directory registry generated from `cargo vendor` because the versions
// referenced from `Cargo.toml` may not exist on crates.io.
fn verify(crates: &[Crate]) {
    drop(fs::remove_dir_all(".cargo"));
    drop(fs::remove_dir_all("vendor"));
    let vendor = Command::new("cargo")
        .arg("vendor")
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    assert!(vendor.status.success());

    fs::create_dir_all(".cargo").unwrap();
    fs::write(".cargo/config.toml", vendor.stdout).unwrap();

    for krate in crates {
        if !krate.publish {
            continue;
        }
        verify_and_vendor(krate);
    }

    fn verify_and_vendor(krate: &Crate) {
        let mut cmd = Command::new("cargo");
        cmd.arg("package")
            .arg("--manifest-path")
            .arg(&krate.path)
            .env("CARGO_TARGET_DIR", "./target");
        let status = cmd.status().unwrap();
        assert!(status.success(), "failed to verify {:?}", &krate.manifest);
        let tar = Command::new("tar")
            .arg("xf")
            .arg(format!(
                "../target/package/{}-{}.crate",
                krate.name, krate.version
            ))
            .current_dir("./vendor")
            .status()
            .unwrap();
        assert!(tar.success());
        fs::write(
            format!(
                "./vendor/{}-{}/.cargo-checksum.json",
                krate.name, krate.version
            ),
            "{\"files\":{}}",
        )
        .unwrap();
    }
}
