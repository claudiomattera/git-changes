// Copyright Claudio Mattera 2021.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use regex::Regex;

use semver::Version;

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

    /// Print changelog for latest version only
    #[structopt(short, long)]
    pub only_last: bool,

    /// Commit message regular expression
    #[structopt(short, long, default_value = r"(.+)\s+\(issue\s+#(\d+)\)")]
    pub commit_regex: Regex,

    /// Commit message replacement text
    #[structopt(short = "r", long, default_value = "${1} (issue ${2})")]
    pub commit_replacement: String,

    /// Add version description from tag messages
    #[structopt(long)]
    pub add_tag_description: bool,

    /// Include the current head as last version
    #[structopt(short, long)]
    pub include_head: Option<Version>,

    /// Set the current head description
    #[structopt(short = "d", long)]
    pub head_description: Option<String>,

    /// Strip GPG signature from version descriptions
    #[structopt(long)]
    pub strip_gpg_signature: bool,

    /// Generate changelog for selected versions
    #[structopt(short = "s", long = "select-version")]
    pub selected_versions: Option<Vec<Version>>,
}
