mod pheader;
mod fplan;

extern crate toml;
extern crate serde;
extern crate serde_json;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use std::fs::{File, read_to_string};
use std::sync::mpsc::channel;
use std::time::Duration;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify::DebouncedEvent::Write;
use crate::fplan::Feature;
use crate::pheader::Project;
use serde::Serialize;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
	#[structopt(short = "w", help = "Keeps watching source API file changes")]
	watch: bool,
	#[structopt(short = "i", help = "Input folder")]
	source: String,
	#[structopt(short = "o", help = "Output file",  default_value = "./plan.json")]
	output_file: String,
}

#[derive(Serialize)]
struct FidoPlan {
	header: Project,
	features: Vec<Feature>,
}

fn get_file_content(file_name: String) -> Result<String, String> {
	match read_to_string(file_name) {
		Ok(content) => Ok(content),
		Err(e) => Err(format!("{}", e)),
	}
}

fn process(from_folder: &str) -> Result<FidoPlan, String>{
	// generating project header
	let header_file = format!("{}/plan-header.toml", from_folder);
	let content = get_file_content(header_file)?;
	let project = pheader::parse_project(content.as_str()).unwrap();

	// generating project detail
	let plan_file = format!("{}/plan.fplan", from_folder);
	let plan_txt = get_file_content(plan_file)?;
	let plan = fplan::get_plan(plan_txt.as_str()).unwrap();

	let f_plan = FidoPlan {
		header: project,
		features: plan,
	};
	Ok(f_plan)
}

fn execute(from_source: &str, target: &str) {
	match process(from_source) {
		Ok(plan) => {
			let plan_file = File::create(target).unwrap();
			serde_json::to_writer(plan_file, &plan).unwrap();
			println!("{} generated", target);
		}
		Err(e) => {
			println!("Cannot process file: {}", e);
		}
	}
}

fn watch(from_source: &str, target: &str) -> notify::Result<()>{
	let (tx, rx) = channel();
	let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;
	watcher.watch(from_source, RecursiveMode::NonRecursive)?;
	loop {
		match rx.recv() {
			Ok(event) => {
				if let Write(_) = event {
					execute(from_source, target);
				}
			}
			Err(e) => {
				println!("Error in source file {:?}", e);
			}
		}
	}
}

fn main() {
	let version = env!("CARGO_PKG_VERSION");
	let opt = Opt::from_args();
	println!("🦀 Planner {}\n\tWatching: {}\n", version, opt.source);
	if opt.watch {
		println!("Watching source files");
		if let Err(e) = watch(opt.source.as_ref(), opt.output_file.as_ref()) {
			println!("Cannot watch folder {}", e);
		}
	} else {
		execute(opt.source.as_ref(), opt.output_file.as_ref());
	}

}
