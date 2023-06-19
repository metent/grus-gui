// This file is part of grus-gui, a hierarchical task management application.
// Copyright (C) 2023 Rishabh Das
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::iter;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display, Formatter};
use chrono::{Datelike, Local, NaiveDateTime};
use grus_lib::{Error, Store};
use grus_lib::types::Session;

#[derive(Default)]
pub struct Node {
	pub id: u64,
	pub name: String,
	pub due_date: Option<NaiveDateTime>,
	pub session: Option<Session>,
}

#[derive(Default)]
pub struct Tree {
	nodes: HashMap<u64, Node>,
	links: HashMap<u64, Vec<u64>>,
	selections: HashMap<u64, HashSet<u64>>,
	pub highlighted: Option<u64>,
}

impl Tree {
	pub fn from_store(store: &Store) -> Result<Tree, Error> {
		let mut tree = Tree::default();
		tree.rebuild(store)?;
		Ok(tree)
	}

	pub fn rebuild(&mut self, store: &Store) -> Result<(), Error> {
		self.nodes.clear();
		self.links.clear();

		let reader = store.reader()?;
		for entry in reader.all_names()? {
			let (&id, name) = entry?;
			self.nodes.insert(id, Node {
				id,
				name: name.to_string().into(),
				due_date: reader.due_date(id)?,
				session: reader.first_session(id)?,
			});
			self.links.insert(id, reader.child_ids(id)?.collect::<Result<_, Error>>()?);
		}
		Ok(())
	}

	pub fn toggle(&mut self, pid: u64, id: u64) {
		if let Some(pids) = self.selections.get_mut(&id) {
			if !pids.insert(pid) {
				pids.remove(&pid);
				if pids.is_empty() {
					self.selections.remove(&id);
				}
			}
		} else {
			self.selections.insert(id, HashSet::from([pid]));
		}
	}

	pub fn node_at(&self, id: u64) -> &Node {
		&self.nodes[&id]
	}

	pub fn children(&self, id: u64) -> impl Iterator<Item = &Node> {
		self.links[&id].iter().map(|id| &self.nodes[&id])
	}

	pub fn selections(&self) -> impl Iterator<Item = (&u64, &u64)> {
		if !self.selections.is_empty() {
			Selections::Actual(self.selections.iter().flat_map(|(id, pids)| pids.iter().zip(iter::repeat(id))))
		} else {
			Selections::Empty(iter::empty())
		}
	}

	pub fn selection_ids(&self) -> impl Iterator<Item = &u64> {
		if !self.selections.is_empty() {
			SelectionIds::Actual(self.selections.keys())
		} else {
			SelectionIds::Empty(iter::empty())
		}
	}

	pub fn is_selected(&self, pid: u64, id: u64) -> bool {
		if let Some(pids) = self.selections.get(&id) {
			pids.contains(&pid)
		} else {
			false
		}
	}
}

pub enum Selections<'s, T: Iterator<Item = (&'s u64, &'s u64)>> {
	Actual(T),
	Empty(iter::Empty<(&'s u64, &'s u64)>),
}

impl<'s, T: Iterator<Item = (&'s u64, &'s u64)>> Iterator for Selections<'s, T> {
	type Item = (&'s u64, &'s u64);

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Selections::Actual(iter) => iter.next(),
			Selections::Empty(iter) => iter.next(),
		}
	}
}

pub enum SelectionIds<'s, T: Iterator<Item = &'s u64>> {
	Actual(T),
	Empty(iter::Empty<&'s u64>),
}

impl<'s, T: Iterator<Item = &'s u64>> Iterator for SelectionIds<'s, T> {
	type Item = &'s u64;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			SelectionIds::Actual(iter) => iter.next(),
			SelectionIds::Empty(iter) => iter.next(),
		}
	}
}

pub struct Displayable<T>(pub Option<T>);

impl Display for Displayable<NaiveDateTime> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let Displayable(Some(dt)) = self else { return Ok(()) };
		let now = Local::now().naive_local();

		if dt.year() != now.year() {
			write!(f, "{}", dt.format("%e %b %Y %-I:%M %p"))
		} else if dt.iso_week() != now.iso_week() {
			write!(f, "{}", dt.format("%e %b %-I:%M %p"))
		} else if dt.day() != now.day() {
			write!(f, "{}", dt.format("%A %-I:%M %p"))
		} else {
			write!(f, "{}", dt.format("%-I:%M %p"))
		}
	}
}

impl Display for Displayable<Session> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let Displayable(Some(session)) = self else { return Ok(()) };

		write!(f, "{} to {}", Displayable(Some(session.start)), Displayable(Some(session.end)))
	}
}
