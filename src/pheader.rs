use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Link {
	pub label: String,
	pub url: String,
}

#[derive(Deserialize, Debug)]
struct RawTeam {
	name: String,
	contact: String,
}

#[derive(Deserialize, Debug)]
struct RawProject {
	title: String,
	variables: Option<HashMap<String, String>>,
	services: HashMap<String, String>,
	resources: HashMap<String, String>,
	iterations: Option<HashMap<String, String>>,
	teams: Option<HashMap<String, RawTeam>>,
	links: Option<Vec<Link>>,
}

#[derive(Serialize, Debug, Ord, Eq, PartialOrd, PartialEq)]
pub struct Record {
	pub key: String,
	pub value: String,
}

/// Project intermediate representation
#[derive(Debug, Serialize)]
pub struct Project {
	pub title: String,
	pub variables: HashMap<String, String>,
	pub services: Vec<Record>,
	pub resources: Vec<Record>,
	pub iterations: Vec<Record>,
	pub teams: Vec<Team>,
	pub links: Vec<Link>,
}

#[derive(Debug, Serialize, Eq, Ord, PartialOrd, PartialEq)]
pub struct Team {
	pub alias: String,
	pub name: String,
	pub contact: String,
}

fn get_teams(from: Option<HashMap<String, RawTeam>>) -> Vec<Team> {
	match from {
		Some(tm) => {
			let mut teams = Vec::new();
			for r in tm.iter() {
				teams.push(Team {
					alias: r.0.to_owned(),
					name: r.1.name.to_owned(),
					contact: r.1.contact.to_owned(),
				})
			}
			teams.sort();
			teams
		}
		None => {
			Vec::new()
		}
	}
}

fn sorted_vector_from_opt(from_map: Option<HashMap<String, String>>) -> Vec<Record> {
	match from_map {
		Some(m) => {
			sorted_vector(m)
		}
		None => {
			Vec::new()
		}
	}
}

fn sorted_vector(from_map: HashMap<String, String>) -> Vec<Record> {
	let mut items = Vec::new();
	for r in from_map.iter() {
		items.push(Record {
			key: r.0.to_owned(),
			value: r.1.to_owned(),
		});
	}
	items.sort();
	items
}

fn get_project(from: &str) -> Result<RawProject, String> {
	let project: Result<RawProject, toml::de::Error > = toml::from_str(from);
	match project {
		Ok(p) => {
			Ok(p)
		},
		Err(e) => {
			Err(e.to_string())
		}
	}
}

/// Returns a project from a toml source
pub fn parse_project(from: &str) -> Result<Project, String> {
	let project = get_project(from)?;
	let variables = if let Some(v) = project.variables {
		v
	} else {
		HashMap::new()
	};
	let links = if let Some(l) = project.links {
		l
	} else {
		Vec::new()
	};
	Ok(Project {
		title: project.title,
		variables,
		services: sorted_vector(project.services),
		resources: sorted_vector(project.resources),
		iterations: sorted_vector_from_opt(project.iterations),
		links,
		teams: get_teams(project.teams),
	})
}