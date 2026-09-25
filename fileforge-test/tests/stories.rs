//! Renders every story in the storybook and compares it against `stories/`.
//!
//! Run with `UPDATE_STORIES=1` to write new or changed renders instead of failing. A story whose
//! renderer panics is recorded as `PANICKED: <message>`, so unfinished renderers show up here
//! rather than stopping the test.

use std::{
  collections::BTreeSet,
  fs,
  path::{Path, PathBuf},
};

use fileforge::{
  error::render::buffer::cell::tag::context::RenderMode,
  storybook::{iter_stories, quietly, render_story, Story},
};

// Stories are registered by the crates that declare them, so every crate must be linked.
extern crate fileforge_nintendo;
extern crate fileforge_std;

const WIDTH: usize = 80;

fn slug(name: &str) -> String {
  let mut slug = String::new();

  for c in name.chars() {
    if c.is_ascii_alphanumeric() {
      slug.push(c.to_ascii_lowercase());
    } else if !slug.ends_with('-') {
      slug.push('-');
    }
  }

  slug.trim_matches('-').to_string()
}

fn snapshot_path(root: &Path, story: &Story) -> PathBuf {
  root.join(story.crate_name()).join(story.type_name).join(format!("{}.txt", slug(story.name)))
}

fn render(story: &Story) -> String {
  render_story(story, RenderMode::PlainText, WIDTH).unwrap_or_else(|message| format!("PANICKED: {message}\n"))
}

fn snapshot_files(dir: &Path, into: &mut BTreeSet<PathBuf>) {
  let Ok(entries) = fs::read_dir(dir) else { return };

  for entry in entries.flatten() {
    let path = entry.path();

    if path.is_dir() {
      snapshot_files(&path, into);
    } else if path.extension().is_some_and(|ext| ext == "txt") {
      into.insert(path);
    }
  }
}

#[test]
fn stories_match_snapshots() {
  let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("stories");
  let update = std::env::var_os("UPDATE_STORIES").is_some();

  let mut stories: Vec<&Story> = iter_stories().collect();
  stories.sort_by_key(|story| (story.module, story.type_name, story.name));

  // Rendering panics are expected for unfinished renderers; keep them out of the test output.
  let rendered: Vec<(&Story, PathBuf, String)> = quietly(|| stories.iter().map(|story| (*story, snapshot_path(&root, story), render(story))).collect());

  let mut problems = Vec::new();
  let mut seen = BTreeSet::new();

  for (story, path, actual) in &rendered {
    if !seen.insert(path.clone()) {
      problems.push(format!("{} / {}: another story on this type has the same file name", story.type_name, story.name));
      continue;
    }

    let expected = fs::read_to_string(path).ok().map(|s| s.replace("\r\n", "\n"));

    if expected.as_deref() == Some(actual.as_str()) {
      continue;
    }

    if update {
      fs::create_dir_all(path.parent().unwrap()).unwrap();
      fs::write(path, actual).unwrap();
      continue;
    }

    match expected {
      None => problems.push(format!("{} / {}: no snapshot at {}", story.type_name, story.name, path.display())),
      Some(expected) => {
        let line = expected.lines().zip(actual.lines()).position(|(e, a)| e != a).unwrap_or(expected.lines().count().min(actual.lines().count()));
        problems.push(format!(
          "{} / {}: differs from {} at line {}\n  expected: {:?}\n  actual:   {:?}",
          story.type_name,
          story.name,
          path.display(),
          line + 1,
          expected.lines().nth(line).unwrap_or("<end of file>"),
          actual.lines().nth(line).unwrap_or("<end of file>"),
        ));
      }
    }
  }

  let mut on_disk = BTreeSet::new();
  snapshot_files(&root, &mut on_disk);

  for stale in on_disk.difference(&seen) {
    if update {
      fs::remove_file(stale).unwrap();
    } else {
      problems.push(format!("{}: no story renders to this file (removed, renamed, or not linked)", stale.display()));
    }
  }

  assert!(
    problems.is_empty(),
    "{} problems across {} stories (rerun with UPDATE_STORIES=1 to accept the new renders):\n\n{}",
    problems.len(),
    rendered.len(),
    problems.join("\n\n")
  );
}
