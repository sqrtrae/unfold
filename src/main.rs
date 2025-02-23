#![doc = include_str!("../README.md")]

use anyhow::{anyhow, bail, Context, Result};
use clap::error::ErrorKind::DisplayHelp;
use clap::Parser;
use std::path::{Path, PathBuf};
use symlink::{remove_symlink_auto, remove_symlink_dir, remove_symlink_file, symlink_auto};

/// Unfold symbolic links to their targets.
///
/// Symbolic links to files are replaced with copies of their targets.
/// Symbolic links to directories are replaced with a directory whose
/// contents are symbolic links to the contents of the targets. In both cases,
/// the names of the original symbolic links are retained by the new files
/// or directories.
///
/// If an error occurs while unfolding a symbolic link, that symbolic link will
/// be reverted to its original state, if possible.
///
/// If multiple symbolic links are given as arguments, the symbolic links
/// will be unfolded in the order they are given, and will only be unfolded if
/// all prior symbolic links were successfully unfolded. Any symbolic link that
/// was successfully unfolded prior to an error will not be reverted.
///
/// By default, symbolic links are unfolded to their immediate targets, which
/// may also be symbolic links. To follow all symbolic links in the chain to
/// the actual source, use the option '-f' or '--follow-to-source'. To follow
/// up to NUM number of symbolic links in the chain, use the option '-n NUM'
/// or '--num-layers NUM'. Note '-n 1' is equivalent to the default behavior,
/// and '-n 0' will do nothing.
#[derive(Debug, Parser)]
#[command(version, about("Unfold symbolic links to their targets."), long_about)]
struct Args {
    /// Symbolic links to unfold.
    #[arg(value_name("SYMLINK"), required(true))]
    symlinks: Vec<PathBuf>,

    /// Follow symbolic links to their source.
    ///
    /// Incompatible with '-n' or '--num-layers'.
    #[arg(short('f'), long("follow-to-source"), conflicts_with("num_layers"))]
    follow_to_source: bool,

    /// Follow up to NUM symbolic links.
    ///
    /// Incompatible with '-f' or '--follow-to-source'.
    #[arg(
        short('n'),
        long("num-layers"),
        value_name("NUM"),
        default_value("1"),
        hide_default_value(true),
        conflicts_with("follow_to_source")
    )]
    num_layers: u8,
}

fn try_absolute_path(path: &PathBuf) -> Result<PathBuf> {
    match path.is_absolute() {
        true => Ok(path.into()),
        false => Ok(std::env::current_dir()
            .context("Current working directory is unreachable.")?
            .join(path)),
    }
}

fn validate_symlink(symlink: &PathBuf) -> Result<()> {
    if !symlink.is_symlink() {
        bail!("{:#?} is not a symlink.", symlink)
    } else if !symlink
        .try_exists()
        .context(format!("{:#?} is unreachable.", symlink))?
    {
        bail!("{:#?} is a broken symlink.", symlink)
    };
    Ok(())
}

fn try_find_target(symlink: &Path, num_layers: u8, follow_to_source: bool) -> Result<PathBuf> {
    if follow_to_source {
        return Ok(symlink.canonicalize()?);
    }

    let mut target = symlink.to_path_buf();
    for _ in 0..num_layers {
        if target.is_symlink() {
            // have to join w/ parent dir because read_link gives a relative path.
            target = target.parent().unwrap().join(target.read_link()?);
        } else {
            break;
        };
    }
    Ok(target)
}

fn try_symlink_unfold(symlink: &PathBuf, target: &PathBuf) -> Result<()> {
    remove_symlink_auto(symlink).context(format!("Could not unlink {:#?}.", symlink))?;
    symlink_auto(try_find_target(target, 1, false)?, symlink).context(format!(
        "Could not copy symlink {:#?} to {:#?}",
        target, symlink
    ))?;
    Ok(())
}

fn try_file_unfold(symlink: &PathBuf, target: &PathBuf) -> Result<()> {
    remove_symlink_file(symlink).context(format!("Could not unlink {:#?}.", symlink))?;
    std::fs::copy(target, symlink).context(format!(
        "Could not copy file {:#?} to {:#?}.",
        target, symlink
    ))?;
    Ok(())
}

fn try_dir_unfold(symlink_dir: &PathBuf, target_dir: &PathBuf) -> Result<()> {
    remove_symlink_dir(symlink_dir).context(format!("Could not unlink {:#?}.", symlink_dir))?;
    std::fs::create_dir(symlink_dir)
        .context(format!("Could not create directory at {:#?}.", symlink_dir))?;
    let children = target_dir
        .read_dir()
        .context(format!("Could not read contents of {:#?}", target_dir))?;
    for child in children {
        let target = &child?.path();
        let symlink = &symlink_dir.join(target.file_name().unwrap());
        symlink_auto(target, symlink)
            .context(format!("Could not symlink {:#?} to {:#?}", target, symlink))?;
    }
    Ok(())
}

fn try_unfold(symlink: &PathBuf, num_layers: u8, follow_to_source: bool) -> Result<()> {
    let target = &try_find_target(symlink, num_layers, follow_to_source)?;

    if target.is_symlink() {
        try_symlink_unfold(symlink, target)?;
    } else if target.is_file() {
        try_file_unfold(symlink, target)?;
    } else if target.is_dir() {
        try_dir_unfold(symlink, target)?;
    } else {
        bail!("Could not unfold {:#?}.", symlink);
    }

    println!(
        "Successfully unfolded {:#?} targeting {:#?}",
        symlink, target,
    );
    Ok(())
}

fn try_revert(symlink: &PathBuf, target: &PathBuf) -> Result<()> {
    let exists = symlink.try_exists()?;
    if exists && symlink.is_file() {
        std::fs::remove_file(symlink)?;
    } else if exists && symlink.is_dir() {
        std::fs::remove_dir_all(symlink)?;
    }
    symlink::symlink_auto(target, symlink)?;
    Ok(())
}

fn main() -> Result<()> {
    // The default error message format for clap is "error: {err}".
    // In contrast, anyhow error messages are prepended with "Error: "
    // when formatted, creating an output of "Error: {err}". To make
    // the capitalization consistent, we strip out the beginning of
    // clap's error message, leaving only "{err}", and then use
    // anyhow to format the error.
    let args = Args::try_parse().map_err(|err| {
        // Help text in clap is an error type, so we need to
        // special-case it when mapping the clap error.
        if err.kind() == DisplayHelp {
            print!("{}", err);
            std::process::exit(0);
        }
        let err_str = err.to_string();
        match err_str.starts_with("error: ") {
            true => anyhow!("{}", err_str.split_at(7).1),
            false => err.into(),
        }
    })?;

    if args.num_layers == 0 {
        println!("Did nothing. :/");
        return Ok(());
    }

    for symlink in args.symlinks {
        let symlink = &try_absolute_path(&symlink)?;
        validate_symlink(symlink)?;
        let target = &try_find_target(symlink, 1, false)?;
        try_unfold(symlink, args.num_layers, args.follow_to_source).or_else(
            |err| match try_revert(symlink, target) {
                Ok(()) => Err(err),
                Err(revert_err) => {
                    Err(err).context(format!("Could not revert {:#?}: {}", symlink, revert_err))
                }
            },
        )?;
    }

    Ok(())
}
