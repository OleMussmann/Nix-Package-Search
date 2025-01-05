use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use predicates::prelude::predicate;
use regex::Regex;
use std::{fs, process::Command};
use tempfile::{NamedTempFile, TempDir};

#[test]
fn short_help() {
    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg("-h");
    cmd.assert().success().stdout(predicate::str::contains(
        "Find SEARCH_TERM in available nix packages and sort results by relevance",
    ));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("experimental"));
}

#[test]
fn long_help() {
    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Use up to four times for increased verbosity",
    ));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Flip the order of matches?"));
}

#[test]
fn no_search_term() {
    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.assert().failure().stderr(predicate::str::contains(
        "error: the following required arguments were not provided:
  <SEARCH_TERM>",
    ));
}

#[test]
fn too_much_debug() {
    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg("-ddddd")
        .arg("search_term")
        .env_clear();
    cmd.assert().failure().stderr(predicate::str::contains(
        "Max log level is 4",
    ));
}

#[test]
fn experimental_output_case_sensitive() {
    let desired_output =
        "MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
MatchMyDescription   a.b.c  MyTestPackageName appears in my description
mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName

MyTestPackageName3   1.2.1  More test package description
MyTestPackageName2   1.0.1  
MyTestPackageName1   1.1.0  Another test package description

MyTestPackageName    1.0.0  Test package description
";
    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg("-i=false")
        .arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn experimental_output() {
    let desired_output = "MatchMyDescription2  9.8.7  mytestpackageName appears in my description with different capitalization
MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
MatchMyDescription   a.b.c  MyTestPackageName appears in my description

mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName
MyTestPackageName3   1.2.1  More test package description
MyTestPackageName2   1.0.1  
MyTestPackageName1   1.1.0  Another test package description

MyTestPackageName    1.0.0  Test package description
";
    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn experimental_output_flip_by_command_line_no_equals(
) {
    let desired_output = "MyTestPackageName    1.0.0  Test package description

MyTestPackageName1   1.1.0  Another test package description
MyTestPackageName2   1.0.1  
MyTestPackageName3   1.2.1  More test package description

mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName
MatchMyDescription   a.b.c  MyTestPackageName appears in my description
MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
";
    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg("-i=false")
        .arg("-f")
        .arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn experimental_output_flip_by_command_line_equals() {
    let desired_output = "MyTestPackageName    1.0.0  Test package description

MyTestPackageName1   1.1.0  Another test package description
MyTestPackageName2   1.0.1  
MyTestPackageName3   1.2.1  More test package description

mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName
MatchMyDescription   a.b.c  MyTestPackageName appears in my description
MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
";
    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg("-i=false")
        .arg("-f=true")
        .arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn experimental_output_flip_by_env_var() {
    let desired_output = "MyTestPackageName    1.0.0  Test package description

MyTestPackageName1   1.1.0  Another test package description
MyTestPackageName2   1.0.1  
MyTestPackageName3   1.2.1  More test package description

mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName
MatchMyDescription   a.b.c  MyTestPackageName appears in my description
MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
";
    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg("-i=false");
    cmd.arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .env_clear()
        .env("NIX_PACKAGE_SEARCH_FLIP", "true");

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn output_case_sensitive() {
    // The cache mixes scenarios for nixos-the-OS and nix-the-package-manager. We test for both at
    // the same time.
    let desired_output =
        "nixos.MatchMyDescription1    9.8.7  Also here MyTestPackageName appears in my description
nixos.MatchMyDescription     a.b.c  MyTestPackageName appears in my description
nixos.mytestpackageName3     3.2.1  More test package description, now with MyTestPackageName
nixpkgs.MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
nixpkgs.MatchMyDescription   a.b.c  MyTestPackageName appears in my description
nixpkgs.mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName

nixos.MyTestPackageName3     1.2.1  More test package description
nixos.MyTestPackageName2     1.0.1  
nixos.MyTestPackageName1     1.1.0  Another test package description
nixpkgs.MyTestPackageName3   1.2.1  More test package description
nixpkgs.MyTestPackageName2   1.0.1  
nixpkgs.MyTestPackageName1   1.1.0  Another test package description

nixos.MyTestPackageName      1.0.0  Test package description
nixpkgs.MyTestPackageName    1.0.0  Test package description
";
    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg("-i=false")
        .arg("--cache-folder=tests/")
        .arg("--experimental=false")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn output() {
    // The cache mixes scenarios for nixos-the-OS and nix-the-package-manager. We test for both at
    // the same time.
    let desired_output = "nixos.MatchMyDescription2    9.8.7  mytestpackageName appears in my description with different capitalization
nixos.MatchMyDescription1    9.8.7  Also here MyTestPackageName appears in my description
nixos.MatchMyDescription     a.b.c  MyTestPackageName appears in my description
nixpkgs.MatchMyDescription2  9.8.7  mytestpackageName appears in my description with different capitalization
nixpkgs.MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
nixpkgs.MatchMyDescription   a.b.c  MyTestPackageName appears in my description

nixos.mytestpackageName3     3.2.1  More test package description, now with MyTestPackageName
nixos.MyTestPackageName3     1.2.1  More test package description
nixos.MyTestPackageName2     1.0.1  
nixos.MyTestPackageName1     1.1.0  Another test package description
nixpkgs.mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName
nixpkgs.MyTestPackageName3   1.2.1  More test package description
nixpkgs.MyTestPackageName2   1.0.1  
nixpkgs.MyTestPackageName1   1.1.0  Another test package description

nixos.MyTestPackageName      1.0.0  Test package description
nixpkgs.MyTestPackageName    1.0.0  Test package description
";
    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg("--cache-folder=tests/")
        .arg("--experimental=false")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

// The following tests are not run by default. Use
// cargo test -- --ignored
// to execute them.
#[test]
#[ignore]
/// Testing the creation of new caches. This requires internet connection, so
/// it is disabled by default.
fn cache_creation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_owned();

    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg(format!("--cache-folder={}", &temp_path.display()))
        .arg("--experimental=false")
        .arg("-r")
        .env_clear(); // remove env vars

    cmd.assert().success();

    let cache_content = fs::read_to_string(&temp_path.join("nps.dev.cache")).unwrap();
    let re = Regex::new("vim .*popular clone of the VI editor").unwrap();
    assert!(re.is_match(&cache_content));
}

#[test]
#[ignore]
/// Testing the creation of new caches. This requires internet connection, so
/// it is disabled by default.
fn experimental_cache_creation() -> () {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_owned();

    let mut cmd = Command::cargo_bin("nps").unwrap();
    cmd.arg(format!("--cache-folder={}", &temp_path.display()))
        .arg("--experimental=true")
        .arg("-r")
        .env_clear(); // remove env vars

    cmd.assert().success();

    let cache_content = fs::read_to_string(&temp_path.join("nps.experimental.dev.cache")).unwrap();
    let re = Regex::new("vim .*popular clone of the VI editor").unwrap();
    assert!(re.is_match(&cache_content));
}
