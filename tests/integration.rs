use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use predicates::prelude::predicate;
use std::process::Command;

#[test]
fn test_short_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("nps")?;
    cmd.arg("-h");
    cmd.assert().success().stdout(predicate::str::contains(
        "Find SEARCH_TERM in available nix packages and sort results by relevance",
    ));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("experimental"));

    Ok(())
}

#[test]
fn test_long_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("nps")?;
    cmd.arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Use up to four times for increased verbosity",
    ));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Flip the order of matches?"));

    Ok(())
}

#[test]
fn test_no_search_term() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("nps")?;
    cmd.assert().failure().stderr(predicate::str::contains(
        "error: the following required arguments were not provided:
  <SEARCH_TERM>",
    ));

    Ok(())
}

#[test]
fn test_experimental_output_case_sensitive() -> Result<(), Box<dyn std::error::Error>> {
    let desired_output =
        "MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
MatchMyDescription   a.b.c  MyTestPackageName appears in my description
mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName

MyTestPackageName3   1.2.1  More test package description
MyTestPackageName2   1.0.1  
MyTestPackageName1   1.1.0  Another test package description

MyTestPackageName    1.0.0  Test package description
";
    let mut cmd = Command::cargo_bin("nps")?;
    cmd.arg("-i=false")
        .arg("--experimental-cache-file=test.experimental.cache")
        .arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));

    Ok(())
}

#[test]
fn test_experimental_output() -> Result<(), Box<dyn std::error::Error>> {
    let desired_output = "MatchMyDescription2  9.8.7  mytestpackageName appears in my description with different capitalization
MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
MatchMyDescription   a.b.c  MyTestPackageName appears in my description

mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName
MyTestPackageName3   1.2.1  More test package description
MyTestPackageName2   1.0.1  
MyTestPackageName1   1.1.0  Another test package description

MyTestPackageName    1.0.0  Test package description
";
    let mut cmd = Command::cargo_bin("nps")?;
    cmd.arg("--experimental-cache-file=test.experimental.cache")
        .arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));

    Ok(())
}

#[test]
fn test_experimental_output_flip_by_command_line_no_equals(
) -> Result<(), Box<dyn std::error::Error>> {
    let desired_output = "MyTestPackageName    1.0.0  Test package description

MyTestPackageName1   1.1.0  Another test package description
MyTestPackageName2   1.0.1  
MyTestPackageName3   1.2.1  More test package description

mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName
MatchMyDescription   a.b.c  MyTestPackageName appears in my description
MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
";
    let mut cmd = Command::cargo_bin("nps")?;
    cmd.arg("-i=false")
        .arg("-f")
        .arg("--experimental-cache-file=test.experimental.cache")
        .arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));

    Ok(())
}

#[test]
fn test_experimental_output_flip_by_command_line_equals() -> Result<(), Box<dyn std::error::Error>>
{
    let desired_output = "MyTestPackageName    1.0.0  Test package description

MyTestPackageName1   1.1.0  Another test package description
MyTestPackageName2   1.0.1  
MyTestPackageName3   1.2.1  More test package description

mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName
MatchMyDescription   a.b.c  MyTestPackageName appears in my description
MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
";
    let mut cmd = Command::cargo_bin("nps")?;
    cmd.arg("-i=false")
        .arg("-f=true")
        .arg("--experimental-cache-file=test.experimental.cache")
        .arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));

    Ok(())
}

#[test]
fn test_experimental_output_flip_by_env_var() -> Result<(), Box<dyn std::error::Error>> {
    let desired_output = "MyTestPackageName    1.0.0  Test package description

MyTestPackageName1   1.1.0  Another test package description
MyTestPackageName2   1.0.1  
MyTestPackageName3   1.2.1  More test package description

mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName
MatchMyDescription   a.b.c  MyTestPackageName appears in my description
MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description
";
    let mut cmd = Command::cargo_bin("nps")?;
    cmd.arg("-i=false");
    cmd.arg("--experimental-cache-file=test.experimental.cache")
        .arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .env_clear()
        .env("NIX_PACKAGE_SEARCH_FLIP", "true");

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));

    Ok(())
}

#[test]
fn test_output_case_sensitive() -> Result<(), Box<dyn std::error::Error>> {
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
    let mut cmd = Command::cargo_bin("nps")?;
    cmd.arg("-i=false")
        .arg("--experimental-cache-file=test.cache")
        .arg("--cache-folder=tests/")
        .arg("--experimental=false")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));

    Ok(())
}

#[test]
fn test_output() -> Result<(), Box<dyn std::error::Error>> {
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
    let mut cmd = Command::cargo_bin("nps")?;
    cmd.arg("--experimental-cache-file=test.cache")
        .arg("--cache-folder=tests/")
        .arg("--experimental=false")
        .arg("MyTestPackageName")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));

    Ok(())
}
