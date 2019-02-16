use std::error::Error;

use colored::*;
use indicatif::ProgressBar;
use log::*;
use vault::prelude::*;

use crate::util::vault_path;

pub(crate) fn add(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let mut context = VaultContext::open(vault_path()?)?;
  let identity = args.value_of("identity").unwrap();
  let force = args.is_present("force");

  let exists = context
    .vault
    .get_identities()
    .contains(&identity.to_string());

  if exists {
    if !force {
      return Err(VaultError::throw("the vault already contains the provided identity, to re-encrypt all entries with this identity, use --force"));
    }
  } else {
    info!("Writing metadata file...");

    context.add_identity(identity);
    context.write()?;
  }

  if !exists || force {
    let index = context.vault.get_index().clone();
    let progress = ProgressBar::new(context.vault.get_index().len() as u64);

    for path in index.keys() {
      let entry = context.read_entry(&path)?;

      context.write_entry(path, &entry)?;

      progress.println(format!("re-encrypting entry {}", path.bold()));
      progress.inc(1);
    }
  }

  Ok(())
}

pub(crate) fn delete(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let mut context = VaultContext::open(vault_path()?)?;
  let identity = args.value_of("identity").unwrap();

  let exists = context
    .vault
    .get_identities()
    .contains(&identity.to_string());

  if !exists {
    return Err(VaultError::throw(
      "the vault does not contain the provided identity",
    ));
  }

  info!("Writing metadata file...");

  context.remove_identity(&identity);
  context.write()?;

  let index = context.vault.get_index().clone();
  let progress = ProgressBar::new(context.vault.get_index().len() as u64);

  for path in index.keys() {
    let entry = context.read_entry(&path)?;

    context.write_entry(path, &entry)?;

    progress.println(format!("re-encrypting entry {}", path.bold()));
    progress.inc(1);
  }

  Ok(())
}