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

use std::ops::Range;
use std::path::Path;
use std::str;
use chrono::{Local, NaiveDateTime};
use eframe::{App, CreationContext, Frame};
use egui::{CentralPanel, Context, FontData, FontDefinitions, FontFamily, FontTweak, TextBuffer, TopBottomPanel};
use egui::text::{CCursor, CCursorRange};
use egui::widgets::TextEdit;
use grus_lib::Store;
use grus_lib::types::Session;
#[cfg(target_os = "android")]
use crate::android::JniWrapper;
use crate::node::Tree;
use crate::ftree::FlatTree;
use grus_gui_lib::datepicker::DatePicker;
use crate::vboard::{Key, VBoard};

pub struct Grus {
	store: Store,
	tree: Tree,
	root_pid: u64,
	root_id: u64,
	stack: Vec<(u64, u64)>,
	todo: Action,
	vboard_text: String,
	vboard_caps: bool,
	start_date: NaiveDateTime,
	end_date: NaiveDateTime,
	#[cfg(target_os = "android")] jniwr: JniWrapper,
}

impl Grus {
	pub fn new<P: AsRef<Path>>(
		path: P,
		n_roots: usize,
		#[cfg(target_os = "android")] jniwr: JniWrapper
	) -> Result<Self, Error> {
		let store = Store::open(path, n_roots)?;
		let tree = Tree::from_store(&store)?;
		Ok(Grus {
			store,
			tree,
			root_pid: 0,
			root_id: 0,
			stack: Vec::new(),
			todo: Action::None,
			vboard_text: "".into(),
			vboard_caps: false,
			start_date: NaiveDateTime::default(),
			end_date: NaiveDateTime::default(),
			#[cfg(target_os = "android")] jniwr,
		})
	}

	pub fn with_scale(self, cc: &CreationContext, ppp: f32) -> Self {
		cc.egui_ctx.set_pixels_per_point(ppp);
		self
	}

	pub fn with_fonts(self, cc: &CreationContext) -> Self {
		let mut fonts = FontDefinitions::default();
		fonts.font_data.insert(
			"Material-Design".into(),
			FontData::from_static(include_bytes!("../assets/materialdesignicons-webfont.ttf")).tweak(
				FontTweak {
					scale: 1.0,
					y_offset_factor: 0.00,
					y_offset: 0.0,
					baseline_offset_factor: 0.0,
				}
			)
		);
		fonts.families.entry(FontFamily::Proportional).or_default().push("Material-Design".into());
		cc.egui_ctx.set_fonts(fonts);
		self
	}

	pub fn perform_action(&mut self, action: Action) -> Result<(), Error> {
		match action {
			Action::Add(_, id) => {
				let mut writer = self.store.writer()?;
				writer.add_child(id, &self.vboard_text)?;
				writer.commit()?;
				self.tree.rebuild(&self.store)?;
				self.vboard_text.clear();
			}
			Action::Delete(pid, id) => {
				let mut writer = self.store.writer()?;
				writer.delete(pid, id)?;
				writer.commit()?;
				self.tree.rebuild(&self.store)?;
			}
			Action::Rename => {
				let mut writer = self.store.writer()?;
				for &id in self.tree.selection_ids() {
					writer.rename(id, &self.vboard_text)?;
				}
				writer.commit()?;
				self.tree.rebuild(&self.store)?;
				self.vboard_text.clear();
			}
			Action::SetDueDate => {
				let mut writer = self.store.writer()?;
				for &id in self.tree.selection_ids() {
					writer.set_due_date(id, self.end_date)?;
				}
				writer.commit()?;
				self.tree.rebuild(&self.store)?;
			}
			Action::AddSession => {
				let mut writer = self.store.writer()?;
				for &id in self.tree.selection_ids() {
					writer.add_session(id, &Session { start: self.start_date, end: self.end_date })?;
				}
				writer.commit()?;
				self.tree.rebuild(&self.store)?;
			}
			Action::Toggle(pid, id) => self.tree.toggle(pid, id),
			Action::Import => {
				#[cfg(target_os = "android")]
				self.jniwr.import()?;
				self.tree.rebuild(&self.store)?;
			}
			Action::Export => {
				#[cfg(target_os = "android")]
				self.jniwr.export()?;
			}
			Action::MoveInto(pid, id) => {
				self.stack.push((self.root_pid, self.root_id));
				self.root_pid = pid;
				self.root_id = id;
				self.tree.rebuild(&self.store)?;
			}
			Action::MoveOut => if let Some((root_pid, root_id)) = self.stack.pop() {
				self.root_pid = root_pid;
				self.root_id = root_id;
				self.tree.rebuild(&self.store)?;
			}
			Action::None => {}
		}
		Ok(())
	}
}

impl App for Grus {
	fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
		let mut action = Action::None;

		TopBottomPanel::top("bar").show_separator_line(false).show(ctx, |ui| {
			ui.add_space(30.0);
			ui.horizontal(|ui| {
				if ui.button("󰁍").clicked() { action = Action::MoveOut }
				if ui.button("󰥝").clicked() { action = Action::Import }
				if ui.button("󰥞").clicked() { action = Action::Export }
				if ui.button("󱰘").clicked() { self.todo = Action::Rename }
				if ui.button("󰃰").clicked() {
					self.end_date = Local::now().naive_local();
					self.todo = Action::SetDueDate
				}
				if ui.button("󰙹").clicked() {
					self.start_date = Local::now().naive_local();
					self.end_date = Local::now().naive_local();
					self.todo = Action::AddSession
				}
			});
		});

		let show_vboard = self.todo != Action::None;
		TopBottomPanel::bottom("vboard").show_animated(ctx, show_vboard, |ui| {
			match self.todo {
				Action::Add(_, _) | Action::Rename => {
					let mut output = TextEdit::singleline(&mut self.vboard_text)
						.desired_width(f32::INFINITY)
						.show(ui);
					let res = if self.vboard_caps {
						ui.caps_vboard()
					} else {
						ui.vboard()
					};
					if let Some(key) = res {
						output.response.request_focus();
						match key {
							Key::Char(c) => {
								if let Some(ccursor_range) = output.state.ccursor_range() {
									let mut ccursor = delete_selected(&mut self.vboard_text, &ccursor_range);
									insert_char(&mut ccursor, &mut self.vboard_text, c);
									output.state.set_ccursor_range(Some(CCursorRange::one(ccursor)));
									output.state.store(ctx, output.response.id);
								} else {
									self.vboard_text.push(c);
								}
							}
							Key::Enter => {
								action = self.todo;
								self.todo = Action::None;
								self.tree.highlighted = None;
							}
							Key::Backspace => {
								if let Some(ccursor_range) = output.state.ccursor_range() {
									let ccursor = if ccursor_range.primary == ccursor_range.secondary {
										delete_previous_char(&mut self.vboard_text, ccursor_range.primary)
									} else {
										delete_selected(&mut self.vboard_text, &ccursor_range)
									};
									output.state.set_ccursor_range(Some(CCursorRange::one(ccursor)));
									output.state.store(ctx, output.response.id);
								}
							}
							Key::CapsLock => {
								self.vboard_caps = !self.vboard_caps;
							}
						}
					}
				}
				Action::SetDueDate => {
					ui.horizontal(|ui| {
						ui.add(DatePicker::<Range<NaiveDateTime>>::new(
							"duedate",
							&mut self.end_date,
						));
						if ui.button("Set").clicked() {
							action = self.todo;
							self.todo = Action::None;
						}
					});
					ui.add_space(200.);
				},
				Action::AddSession => {
					ui.horizontal(|ui| {
						ui.add(DatePicker::<Range<NaiveDateTime>>::new(
							"startdate",
							&mut self.start_date,
						));
						ui.add(DatePicker::<Range<NaiveDateTime>>::new(
							"enddate",
							&mut self.end_date,
						));
						if ui.button("Set").clicked() {
							action = self.todo;
							self.todo = Action::None;
						}
					});
					ui.add_space(200.);
				}
				_ => unreachable!(),
			}
			ui.add_space(30.0);
		});

		CentralPanel::default().show(ctx, |ui| {
			match ui.flattree(&self.tree, self.root_pid, self.root_id) {
				Action::Add(pid, id) => {
					self.todo = Action::Add(pid, id);
					self.tree.highlighted = Some(id);
				}
				Action::Delete(pid, id) => action = Action::Delete(pid, id),
				Action::Toggle(pid, id) => action = Action::Toggle(pid, id),
				Action::MoveInto(pid, id) => action = Action::MoveInto(pid, id),
				Action::MoveOut => action = Action::MoveOut,
				_ => {}
			}
		});

		self.perform_action(action).unwrap();
	}
}

#[derive(Copy, Clone, PartialEq)]
pub enum Action {
	Add(u64, u64),
	Delete(u64, u64),
	Rename,
	SetDueDate,
	AddSession,
	Toggle(u64, u64),
	MoveInto(u64, u64),
	MoveOut,
	Import,
	Export,
	None,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Store Error: {0}")]
	StoreError(#[from] grus_lib::Error),
	#[cfg(target_os = "android")]
	#[error("JNI Error: {0}")]
	JniError(#[from] jni::errors::Error),
}

fn insert_char(
	ccursor: &mut CCursor,
	text: &mut dyn TextBuffer,
	ch: char,
) {
	let mut s = [0; 1];
	ch.encode_utf8(&mut s);
	let text_to_insert = str::from_utf8(&s).unwrap();
	ccursor.index += text.insert_text(text_to_insert, ccursor.index);
}

fn delete_selected(text: &mut dyn TextBuffer, cursor_range: &CCursorRange) -> CCursor {
	let [min, max] = cursor_range.sorted();
	delete_selected_ccursor_range(text, [min, max])
}

fn delete_selected_ccursor_range(text: &mut dyn TextBuffer, [min, max]: [CCursor; 2]) -> CCursor {
	text.delete_char_range(min.index..max.index);
	CCursor {
		index: min.index,
		prefer_next_row: true,
	}
}

fn delete_previous_char(text: &mut dyn TextBuffer, ccursor: CCursor) -> CCursor {
	if ccursor.index > 0 {
		let max_ccursor = ccursor;
		let min_ccursor = max_ccursor - 1;
		delete_selected_ccursor_range(text, [min_ccursor, max_ccursor])
	} else {
		ccursor
	}
}
