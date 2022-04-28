use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Record {
	pub label: String,
	pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Dependency {
	pub team_alias: String,
	pub by: Option<String>,
	// when is it needed by
	pub description: String,
}

#[derive(Debug, Serialize)]
pub struct Effort {
	pub pds: isize,
	pub service: String,
}

#[derive(Debug, Serialize)]
pub struct Resource {
	pub name: String,
	pub when: String,
}

#[derive(Debug, Serialize)]
pub struct Task {
	pub title: String,
	pub links: Vec<Record>,
	pub by: Option<Resource>,
	// alias 1 (as sprint 1)
	pub effort: Option<Effort>,
	pub notes: Vec<String>,
	pub ticket: Option<String>,
	pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Serialize)]
pub struct Feature {
	pub title: String,
	pub links: Vec<Record>,
	pub tasks: Vec<Task>,
}

#[derive(Debug)]
struct Block {
	start: usize,
	end: usize,
}

#[derive(Debug, Ord, Eq, PartialOrd, PartialEq, Copy, Clone, Serialize)]
enum Context {
	Unknown,
	Documents,
	Links,
	Task,
	Effort,
	By,
	Notes,
	Ticket,
	Dependency,
}

fn get_link(from_line: &str) -> Option<Record> {
	let sections: Vec<&str> = from_line.split(": ").collect();
	if sections.len() == 2 {
		let label = sections[0].replace("-", "").trim().to_owned();
		return Some(Record {
			label,
			value: sections[1].to_owned(),
		});
	}
	None
}

fn get_dependency(from: &str) -> Option<Dependency> {
	let words: Vec<&str> = from.trim().split(" ").collect();
	if words.len() < 2 {
		return None;
	}
	Some(Dependency {
		team_alias: words[0].to_owned(),
		by: Some(words[1].to_owned()),
		description: words[2].to_owned(),
	})
}

fn get_effort(from: &str) -> Option<Effort> {
	let s = from.replace("effort:", "").trim().to_owned();
	let v: Vec<&str> = s.split(" ").collect();
	let service = v[0].to_owned();
	let mut pds = "";
	if v.len() > 1 {
		pds = v[1];
	}
	let pds_count = if let Ok(r) = pds.parse::<isize>() {
		r
	} else {
		0
	};
	Some(Effort {
		service,
		pds: pds_count,
	})
}

fn get_by(from: &str) -> Option<Resource> {
	let s = from.replace("by:", "").trim().to_owned();
	let v: Vec<&str> = s.split(" ").collect();
	let name = v[0].to_owned();
	let mut when = "";
	if v.len() > 1 {
		when = v[1];
	}
	Some(Resource {
		name,
		when: when.to_owned(),
	})
}

fn get_task_v2(task_title: &str, from: &Vec<&str>) -> Option<Task> {
	let mut task = Task {
		title: task_title.to_owned(),
		links: vec![],
		by: None,
		effort: None,
		notes: vec![],
		ticket: None,
		dependencies: Vec::new(),
	};
	// notes, links, and dependencies
	let mut notes = Vec::new();
	let mut links = Vec::new();
	let mut dependencies = Vec::new();
	let mut last_context = Context::Unknown;
	for line in from {
		let mut is_keyword = false;
		if line.starts_with("effort:") {
			last_context = Context::Effort;
			task.effort = get_effort(&line);
			is_keyword = true;
		}

		if line.starts_with("ticket:") {
			last_context = Context::Ticket;
			task.ticket = Some(line.replace("ticket:", "").trim().to_string());
			is_keyword = true;
		}

		if line.starts_with("by:") {
			last_context = Context::By;
			task.by = get_by(&line);
			is_keyword = true;
		}

		if line.starts_with("notes:") {
			is_keyword = true;
			last_context = Context::Notes;
		}

		if line.starts_with("links:") {
			is_keyword = true;
			last_context = Context::Links;
		}

		if line.starts_with("dependencies:") {
			is_keyword = true;
			last_context = Context::Dependency;
		}

		if last_context == Context::Notes && !is_keyword {
			notes.push(line.to_string());
		}

		if last_context == Context::Links && !is_keyword {
			if let Some(l) = get_link(&line) {
				links.push(l);
			}
		}

		if last_context == Context::Dependency && !is_keyword {
			if let Some(d) = get_dependency(&line) {
				dependencies.push(d);
			}
		}
	}
	if task.ticket == None {
		task.ticket = Some("".to_string());
	}
	task.notes = notes;
	task.links = links;
	task.dependencies = dependencies;
	Some(task)
}

fn extract_task(lines_source: &Vec<&str>) -> Option<Task> {
	if lines_source.len() == 0 {
		return None;
	}
	let title = lines_source[0].replace("task:", "").trim().to_owned();
	return get_task_v2(&title, &lines_source[1..].to_vec());
}

fn extract_feature(lines_source: &Vec<&str>) -> Option<Feature> {
	if lines_source.len() == 0 {
		return None;
	}
	let title = lines_source[0].replace("feature:", "").trim().to_owned();
	let mut task_start_indices = Vec::new();
	let mut track_docs = true;
	let mut links: Vec<Record> = Vec::new();
	for i in 1..lines_source.len() {
		let line = lines_source[i];
		if line.starts_with("task:") {
			track_docs = false;
			task_start_indices.push(i);
		}
		if track_docs {
			if line.starts_with("docs:") {
				continue;
			}
			if let Some(link) = get_link(&line) {
				links.push(link);
			}
		}
	}
	task_start_indices.push(lines_source.len());
	let limit = task_start_indices.len() - 1;
	let mut tasks = Vec::new();
	for i in 0..limit {
		let first = task_start_indices[i];
		let last = task_start_indices[i + 1];
		let task_lines: Vec<&str> = lines_source[first..last].to_vec();
		if let Some(t) = extract_task(&task_lines) {
			tasks.push(t);
		}
	}
	Some(Feature {
		title,
		links,
		tasks,
	})
}

fn extract_features(from: Vec<&str>) -> Vec<Feature> {
	let blocks = get_feature_blocks(&from);
	let mut features = Vec::new();
	for block in blocks {
		let lines: Vec<&str> = from[block.start..block.end].to_vec();
		if let Some(feat) = extract_feature(&lines) {
			features.push(feat);
		}
	}
	features
}

fn get_feature_blocks(from: &Vec<&str>) -> Vec<Block> {
	let mut feature_indexes = Vec::new();
	let mut index = 0;
	for line in from {
		if line.starts_with("feature:") {
			feature_indexes.push(index);
		}
		index = index + 1;
	}
	let mut blocks = Vec::new();
	for i in 1..feature_indexes.len() {
		blocks.push(Block {
			start: feature_indexes[i - 1],
			end: feature_indexes[i],
		});
	}
	blocks.push(Block {
		start: feature_indexes[feature_indexes.len() - 1],
		end: from.len(),
	});
	blocks
}

fn clear_content(from: &str) -> Vec<&str> {
	let mut target: Vec<&str> = Vec::new();
	for line in from.lines() {
		let trimmed = line.trim();
		if trimmed.len() == 0 || trimmed.starts_with("//") {
			continue;
		}
		target.push(trimmed);
	}
	target
}

/// Converts the plan text file
pub fn get_plan(from: &str) -> Result<Vec<Feature>, String> {
	let features = extract_features(clear_content(from));
	Ok(features)
}

