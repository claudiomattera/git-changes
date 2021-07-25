// Copyright Claudio Mattera 2021.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::stderr;

use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::fmt as subscriber_fmt;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn setup_logging(verbosity: u8) {
    // Redirect all `log`'s events to our subscriber
    LogTracer::init().expect("Failed to set logger");

    let default_log_filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "info,git_changes=debug",
        3 => "info,git_changes=trace",
        4 => "debug",
        _ => "trace",
    };
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_log_filter));

    let formatting_layer = subscriber_fmt::layer()
        .with_target(false)
        .without_time()
        .with_writer(stderr);

    let subscriber = Registry::default().with(env_filter).with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");
}
