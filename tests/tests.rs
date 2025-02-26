use anyhow::Result;
use assert_cmd::Command;
use dircpy::copy_dir;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const CONTENT_PATH: &str = "tests/test_resources/media";
const PERCY_JACKSON_BOOK: &str =
    "media/books 📖/fiction/Percy Jackson and the Olympians - The Lightning Thief";
const MARTIAN_BOOK: &str = "media/books 📖/fiction/The Martian";
const GEORGE_ORWELL_BOOK: &str = "media/books 📖/non-fiction/1984";
const HOWARD_ZINN_BOOK: &str = "media/books 📖/non-fiction/A People's History of the United States";
const MATRIX_MOVIE: &str = "media/movies 📽/The Matrix";
const WALL_E_MOVIE: &str = "media/movies 📽/WALL·E";

const ALL_FILES: [&str; 6] = [
    PERCY_JACKSON_BOOK,
    MARTIAN_BOOK,
    GEORGE_ORWELL_BOOK,
    HOWARD_ZINN_BOOK,
    MATRIX_MOVIE,
    WALL_E_MOVIE,
];

struct TestEnvironment {
    working_dir: TempDir,
}

impl TestEnvironment {
    fn new() -> TestEnvironment {
        let working_dir = tempfile::tempdir().unwrap();
        if let Err(err) = copy_dir(CONTENT_PATH, working_dir.path().join("media")) {
            eprintln!("{}", err);
        }
        TestEnvironment { working_dir }
    }

    fn root(&self) -> &Path {
        self.working_dir.path()
    }

    fn get_full_path<P: AsRef<Path>>(&self, local_path: P) -> PathBuf {
        self.working_dir.path().join(local_path)
    }

    fn create_symlink_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        symlink: P,
        target: Q,
    ) -> Result<()> {
        symlink::symlink_file(self.get_full_path(target), self.get_full_path(symlink))?;
        Ok(())
    }

    fn create_symlink_dir<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        symlink: P,
        target: Q,
    ) -> Result<()> {
        symlink::symlink_dir(self.get_full_path(target), self.get_full_path(symlink))?;
        Ok(())
    }

    fn is_symlink<P: AsRef<Path>>(&self, local_path: P) -> bool {
        self.get_full_path(local_path).is_symlink()
    }

    fn is_file<P: AsRef<Path>>(&self, local_path: P) -> bool {
        self.get_full_path(local_path).is_file()
    }

    fn is_dir<P: AsRef<Path>>(&self, local_path: P) -> bool {
        self.get_full_path(local_path).is_dir()
    }

    fn read_to_string<P: AsRef<Path>>(&self, local_path: P) -> Result<String> {
        let contents = std::fs::read_to_string(self.get_full_path(local_path))?;
        Ok(contents)
    }
}

#[test]
fn test_help_output() -> Result<()> {
    let mut cmd = Command::cargo_bin("unfold")?;
    for arg in ["-h", "--help"] {
        for text in [
            "Usage: unfold [OPTIONS] <SYMLINK>...",
            "Arguments:",
            "Options:",
            "-f, --follow-to-source",
            "-n, --num-layers",
            "-h, --help",
            "-V, --version",
        ] {
            cmd.arg(arg)
                .assert()
                .stdout(predicates::str::contains(text));
        }
    }
    Ok(())
}

#[test]
fn validate_test_environment() -> Result<()> {
    let test_env = TestEnvironment::new();
    for local_path in ALL_FILES {
        let path = test_env.get_full_path(local_path);
        assert!(path.is_file() & !path.is_symlink());
    }
    Ok(())
}

#[test]
fn symlink_to_file() -> Result<()> {
    let test_env = TestEnvironment::new();
    let symlink = "symlink_file";
    test_env.create_symlink_file(symlink, PERCY_JACKSON_BOOK)?;

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .arg(symlink)
        .assert()
        .success()
        .stdout(predicates::str::is_empty());

    assert!(test_env.is_file(symlink) & !test_env.is_symlink(symlink));
    assert_eq!(
        test_env.read_to_string(symlink)?,
        test_env.read_to_string(PERCY_JACKSON_BOOK)?,
    );
    Ok(())
}

#[test]
fn symlink_to_dir() -> Result<()> {
    let test_env = TestEnvironment::new();
    let symlink = "symlink_dir";
    test_env.create_symlink_dir(symlink, "media/movies 📽")?;

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .arg(symlink)
        .assert()
        .success();

    assert!(test_env.is_dir(symlink) & !test_env.is_symlink(symlink));
    for child in test_env.get_full_path(symlink).read_dir()? {
        let child_symlink = &child?.path();
        let child_target = test_env
            .get_full_path("media/movies 📽")
            .join(child_symlink.file_name().unwrap());
        assert!(child_symlink.is_symlink());
        assert_eq!(child_symlink.read_link()?, child_target,);
    }
    Ok(())
}

#[test]
fn follow_to_source() -> Result<()> {
    let test_env = TestEnvironment::new();
    let mut target = PathBuf::from(GEORGE_ORWELL_BOOK);
    for i in 0..5 {
        let symlink = PathBuf::from(format!("symlink_file{}", i + 1));
        test_env.create_symlink_file(&symlink, target)?;
        target = symlink;
    }

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .arg("-f")
        .arg(&target)
        .assert()
        .success();

    assert!(test_env.is_file(&target) & !test_env.is_symlink(&target));
    assert_eq!(
        test_env.read_to_string(&target)?,
        test_env.read_to_string(GEORGE_ORWELL_BOOK)?,
    );
    Ok(())
}

#[test]
fn num_layers_0() -> Result<()> {
    let test_env = TestEnvironment::new();
    let mut target = PathBuf::from(GEORGE_ORWELL_BOOK);
    for i in 0..5 {
        let symlink = PathBuf::from(format!("symlink_file{}", i + 1));
        test_env.create_symlink_file(&symlink, target)?;
        target = symlink;
    }

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .args(["-n", "0"])
        .arg(&target)
        .assert()
        .success();

    assert!(test_env.is_file(&target) & test_env.is_symlink(&target));
    assert_eq!(
        test_env.get_full_path(&target).read_link()?,
        test_env.get_full_path("symlink_file4"),
    );
    Ok(())
}

#[test]
fn num_layers_1() -> Result<()> {
    let test_env = TestEnvironment::new();
    let mut target = PathBuf::from(GEORGE_ORWELL_BOOK);
    for i in 0..5 {
        let symlink = PathBuf::from(format!("symlink_file{}", i + 1));
        test_env.create_symlink_file(&symlink, target)?;
        target = symlink;
    }

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .args(["-n", "1"])
        .arg(&target)
        .assert()
        .success();

    assert!(test_env.is_file(&target) & test_env.is_symlink(&target));
    assert_eq!(
        test_env.get_full_path(&target).read_link()?,
        test_env.get_full_path("symlink_file3"),
    );
    Ok(())
}

#[test]
fn num_layers_2() -> Result<()> {
    let test_env = TestEnvironment::new();
    let mut target = PathBuf::from(GEORGE_ORWELL_BOOK);
    for i in 0..5 {
        let symlink = PathBuf::from(format!("symlink_file{}", i + 1));
        test_env.create_symlink_file(&symlink, target)?;
        target = symlink;
    }

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .args(["-n", "2"])
        .arg(&target)
        .assert()
        .success();

    assert!(test_env.is_file(&target) & test_env.is_symlink(&target));
    assert_eq!(
        test_env.get_full_path(&target).read_link()?,
        test_env.get_full_path("symlink_file2"),
    );
    Ok(())
}

#[test]
fn num_layers_3() -> Result<()> {
    let test_env = TestEnvironment::new();
    let mut target = PathBuf::from(GEORGE_ORWELL_BOOK);
    for i in 0..5 {
        let symlink = PathBuf::from(format!("symlink_file{}", i + 1));
        test_env.create_symlink_file(&symlink, target)?;
        target = symlink;
    }

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .args(["-n", "3"])
        .arg(&target)
        .assert()
        .success();

    assert!(test_env.is_file(&target) & test_env.is_symlink(&target));
    assert_eq!(
        test_env.get_full_path(&target).read_link()?,
        test_env.get_full_path("symlink_file1"),
    );
    Ok(())
}

#[test]
fn num_layers_exceeds_chain_length() -> Result<()> {
    let test_env = TestEnvironment::new();
    let mut target = PathBuf::from(GEORGE_ORWELL_BOOK);
    for i in 0..5 {
        let symlink = PathBuf::from(format!("symlink_file{}", i + 1));
        test_env.create_symlink_file(&symlink, target)?;
        target = symlink;
    }

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .args(["-n", "99"])
        .arg(&target)
        .assert()
        .success();

    assert!(test_env.is_file(&target) & !test_env.is_symlink(&target));
    assert_eq!(
        test_env.read_to_string(&target)?,
        test_env.read_to_string(GEORGE_ORWELL_BOOK)?,
    );
    Ok(())
}

#[test]
fn multiple_symlink_args() -> Result<()> {
    let test_env = TestEnvironment::new();
    let symlink = "symlink_dir";
    test_env.create_symlink_dir(symlink, "media/movies 📽")?;

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .arg(symlink)
        .arg(PathBuf::from(symlink).join("The Matrix"))
        .assert()
        .success();

    assert!(test_env.is_dir(symlink) & !test_env.is_symlink(symlink));
    for child in test_env.get_full_path(symlink).read_dir()? {
        let child_symlink = &child?.path();
        if child_symlink.file_name().unwrap() == "The Matrix" {
            assert!(test_env.is_file(child_symlink) & !test_env.is_symlink(child_symlink));
            assert_eq!(
                test_env.read_to_string(child_symlink)?,
                test_env.read_to_string(MATRIX_MOVIE)?,
            );
        } else {
            let child_target = test_env
                .get_full_path("media/movies 📽")
                .join(child_symlink.file_name().unwrap());
            assert!(child_symlink.is_symlink());
            assert_eq!(child_symlink.read_link()?, child_target,);
        }
    }
    Ok(())
}

#[test]
fn multiple_symlink_args_follow_to_source() -> Result<()> {
    let test_env = TestEnvironment::new();

    let mut target_file = PathBuf::from(MARTIAN_BOOK);
    let mut target_dir = PathBuf::from("media/movies 📽");

    for i in 0..5 {
        let symlink_file = PathBuf::from(format!("symlink_file{}", i + 1));
        let symlink_dir = PathBuf::from(format!("symlink_dir{}", i + 1));
        test_env.create_symlink_file(&symlink_file, target_file)?;
        target_file = symlink_file;
        test_env.create_symlink_dir(&symlink_dir, target_dir)?;
        target_dir = symlink_dir;
    }

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .arg("-f")
        .args([&target_file, &target_dir])
        .assert()
        .success();

    assert!(test_env.is_file(&target_file) & !test_env.is_symlink(&target_file));
    assert_eq!(
        test_env.read_to_string(&target_file)?,
        test_env.read_to_string(MARTIAN_BOOK)?,
    );

    assert!(test_env.is_dir(&target_dir) & !test_env.is_symlink(&target_dir));
    for child in test_env.get_full_path(&target_dir).read_dir()? {
        let child_symlink = &child?.path();
        let child_target = test_env
            .get_full_path("media/movies 📽")
            .join(child_symlink.file_name().unwrap());
        assert!(child_symlink.is_symlink());
        assert_eq!(child_symlink.read_link()?, child_target,);
    }
    Ok(())
}

#[test]
fn multiple_symlink_args_num_layers_3() -> Result<()> {
    let test_env = TestEnvironment::new();

    let mut target_file = PathBuf::from(MARTIAN_BOOK);
    let mut target_dir = PathBuf::from("media/movies 📽");

    for i in 0..5 {
        let symlink_file = PathBuf::from(format!("symlink_file{}", i + 1));
        let symlink_dir = PathBuf::from(format!("symlink_dir{}", i + 1));
        test_env.create_symlink_file(&symlink_file, target_file)?;
        target_file = symlink_file;
        // we only create a chain of length 2 for the symlink dir to test
        // that the num layers argument behaves appropriately w.r.t. each
        // argument.
        if i < 2 {
            test_env.create_symlink_dir(&symlink_dir, target_dir)?;
            target_dir = symlink_dir;
        }
    }

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .args(["-n", "3"])
        .args([&target_file, &target_dir])
        .assert()
        .success();

    assert!(test_env.is_file(&target_file) & test_env.is_symlink(&target_file));
    assert_eq!(
        test_env.get_full_path(&target_file).read_link()?,
        test_env.get_full_path("symlink_file1"),
    );

    assert!(test_env.is_dir(&target_dir) & !test_env.is_symlink(&target_dir));
    for child in test_env.get_full_path(&target_dir).read_dir()? {
        let child_symlink = &child?.path();
        let child_target = test_env
            .get_full_path("media/movies 📽")
            .join(child_symlink.file_name().unwrap());
        assert!(child_symlink.is_symlink());
        assert_eq!(child_symlink.read_link()?, child_target,);
    }
    Ok(())
}

#[test]
fn multiple_symlink_args_revert() -> Result<()> {
    let test_env = TestEnvironment::new();
    let symlink_file = PathBuf::from("symlink_file");
    let symlink_dir = PathBuf::from("symlink_dir");
    test_env.create_symlink_file(&symlink_file, MARTIAN_BOOK)?;
    test_env.create_symlink_dir(&symlink_dir, "media/movies 📽")?;

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .arg("-f")
        // the symlink file should be unfolded, but not the symlink dir due
        // to the erroneous argument between the two.
        .args([
            &symlink_file,
            &PathBuf::from("does_not_exist"),
            &symlink_dir,
        ])
        .assert()
        .failure();

    assert!(test_env.is_file(&symlink_file) & !test_env.is_symlink(&symlink_file));
    assert_eq!(
        test_env.read_to_string(&symlink_file)?,
        test_env.read_to_string(MARTIAN_BOOK)?,
    );

    assert!(test_env.is_dir(&symlink_dir) & test_env.is_symlink(&symlink_dir));
    assert_eq!(
        test_env.get_full_path(&symlink_dir).read_link()?,
        test_env.get_full_path("media/movies 📽"),
    );
    Ok(())
}

#[test]
fn invalid_num_layers() -> Result<()> {
    let test_env = TestEnvironment::new();
    let symlink = "symlink_file";
    test_env.create_symlink_file(symlink, WALL_E_MOVIE)?;

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .args(["-n", "256"])
        .arg(symlink)
        .assert()
        .failure();
    Ok(())
}

#[test]
fn follow_to_source_and_num_layers_conflict() -> Result<()> {
    let test_env = TestEnvironment::new();
    let symlink = "symlink_file";
    test_env.create_symlink_file(symlink, WALL_E_MOVIE)?;

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .args(["-f", "-n", "5"])
        .arg(symlink)
        .assert()
        .failure();
    Ok(())
}

#[test]
fn path_does_not_exist() -> Result<()> {
    let test_env = TestEnvironment::new();

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .arg("does_not_exist")
        .assert()
        .failure();
    Ok(())
}

#[test]
fn path_is_not_a_symlink_file() -> Result<()> {
    let test_env = TestEnvironment::new();

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .arg(MATRIX_MOVIE)
        .assert()
        .failure();
    Ok(())
}

#[test]
fn path_is_not_a_symlink_dir() -> Result<()> {
    let test_env = TestEnvironment::new();

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .arg("media/books 📖")
        .assert()
        .failure();
    Ok(())
}

#[test]
fn path_is_a_broken_symlink() -> Result<()> {
    let test_env = TestEnvironment::new();
    let symlink = "symlink_file";
    test_env.create_symlink_file(symlink, WALL_E_MOVIE)?;
    std::fs::remove_file(test_env.get_full_path(WALL_E_MOVIE))?;

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .arg(symlink)
        .assert()
        .failure();
    Ok(())
}

#[test]
fn verbose_output() -> Result<()> {
    let test_env = TestEnvironment::new();
    let symlink = "symlink_file";
    test_env.create_symlink_file(symlink, PERCY_JACKSON_BOOK)?;

    let mut cmd = Command::cargo_bin("unfold")?;
    cmd.current_dir(test_env.root())
        .arg("-v")
        .arg(symlink)
        .assert()
        .success()
        .stdout(predicates::str::contains(PERCY_JACKSON_BOOK));

    assert!(test_env.is_file(symlink) & !test_env.is_symlink(symlink));
    assert_eq!(
        test_env.read_to_string(symlink)?,
        test_env.read_to_string(PERCY_JACKSON_BOOK)?,
    );
    Ok(())
}
