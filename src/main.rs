// Copyright 2016 The xi-editor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A xi-editor plugin that runs rustfmt on save.

extern crate xi_core_lib as xi_core;
extern crate xi_plugin_lib;
extern crate xi_rope;
extern crate xi_trace;

use std::path::{Path, PathBuf};

use std::process::Command;
use xi_core::ConfigTable;
use xi_plugin_lib::{mainloop, ChunkCache, Plugin, View};
use xi_rope::diff::{Diff, LineHashDiff};
use xi_rope::rope::RopeDelta;
use xi_rope::Rope;
use xi_trace::trace_block;

struct Rustfmt;

impl Plugin for Rustfmt {
    type Cache = ChunkCache;

    fn new_view(&mut self, _view: &mut View<Self::Cache>) {}

    fn did_close(&mut self, _view: &View<Self::Cache>) {}

    fn did_save(&mut self, view: &mut View<Self::Cache>, _old: Option<&Path>) {
        if view.get_language_id().as_ref() == "Rust" {
            if let Err(e) = self.run_rustfmt(view) {
                eprintln!("rustfmt error: {}", e);
            }
        }
    }

    fn config_changed(&mut self, _view: &mut View<Self::Cache>, _changes: &ConfigTable) {}

    fn update(&mut self, _: &mut View<Self::Cache>, _: Option<&RopeDelta>, _: String, _: String) {}
}

impl Rustfmt {
    fn run_rustfmt(&self, view: &mut View<ChunkCache>) -> Result<(), String> {
        let _t = trace_block("Rustfmt::run_rustfmt", &["rustfmt"]);
        let path = view.get_path().map(PathBuf::from).unwrap();
        let output =
            Command::new("rustfmt").args(&["--quiet", "--emit", "stdout"]).arg(&path).output();

        let output = match output {
            Ok(mut output) => {
                if output.status.success() {
                    String::from_utf8(output.stdout).expect("rustfmt output not utf-8?")
                } else {
                    let s = String::from_utf8_lossy(&output.stderr);
                    return Err(format!(
                        "rustfmt exited with code {:?}: '{:?}'",
                        output.status.code(),
                        s
                    ));
                }
            }
            Err(e) => return Err(e.to_string()),
        };

        let base = Rope::from(view.get_document().unwrap());
        let formatted = Rope::from(output);
        let delta = LineHashDiff::compute_delta(&base, &formatted);
        view.edit(delta, 1000, false, true, "rustfmt".into());
        Ok(())
    }
}

fn main() {
    let output = Command::new("rustfmt").arg("--version").output();
    match output {
        Err(e) => {
            eprintln!("error executing rustfmt. Is rustfmt installed?: '{:?}'", e);
            ::std::process::exit(1);
        }
        Ok(output) => {
            let s = String::from_utf8_lossy(&output.stdout);
            eprintln!("using rustfmt {}", s);
        }
    }
    let mut plugin = Rustfmt;
    mainloop(&mut plugin).unwrap();
}
