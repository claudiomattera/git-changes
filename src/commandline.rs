// Copyright Claudio Mattera 2021.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use structopt::clap::{crate_authors, crate_description, crate_name};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = crate_name!(), about = crate_description!(), author = crate_authors!())]
pub struct Arguments {
    /// Verbosity
    #[structopt(short, long = "verbose", parse(from_occurrences))]
    pub verbosity: u8,

    /// Repository path
    #[structopt(parse(from_os_str))]
    pub repo_path: PathBuf,

    /// Only last version changes
    #[structopt(short, long)]
    pub only_last: bool,
}
