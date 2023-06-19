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

use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::vec::IntoIter;
use egui::{Color32, Pos2, RichText, Sense, Ui};
use grus_gui_lib::{Button, Create, Checkbox, ExtLayout, Label, LaidOutButton, LaidOutCheckbox, LaidOutLabel, Paint, WidgetPlacer};
use crate::app::Action;
use crate::node::{Displayable, Node, Tree};

const INDENT_SPACING: f32 = 14.0;

pub trait FlatTree {
	fn flattree(&mut self, tree: &Tree, pid: u64, id: u64) -> Action;
}

impl FlatTree for Ui {
	fn flattree(&mut self, tree: &Tree, pid: u64, id: u64) -> Action {
		let mut wp = WidgetPlacer::new(&self);
		let mut lofnodes = Vec::new();
		let mut queue = VecDeque::new();
		let mut start = 0;
		let maxy = self.available_rect_before_wrap().bottom();

		let root = FNode {
			node: tree.node_at(id),
			path: vec![0],
			pid,
			depth: 0,
			selected: tree.is_selected(pid, id),
			priority: Priority { det: 0, total: 1 },
		};

		let lofnode = create_fnode(&mut wp, root, tree.highlighted.is_some_and(|h| h == id));
		if wp.next_widget_position().y > maxy { return Action::None };
		lofnodes.push(lofnode);

		'outer: loop {
			for i in start..lofnodes.len() {
				queue.push_back(FChildIter::new(&lofnodes[i].fnode, tree));
			}
			start = lofnodes.len();

			while let Some(mut children) = queue.pop_front() {
				let Some(mut child) = children.iter.next() else { continue };
				child.path.push(lofnodes.len());
				let id = child.node.id;
				let lofnode = create_fnode(&mut wp, child, tree.highlighted.is_some_and(|h| h == id));
				if wp.next_widget_position().y > maxy { break 'outer }
				queue.push_back(children);
				lofnodes.push(lofnode);
			}
			if start == lofnodes.len() { break };
		}

		lofnodes.sort_by(|l, r| l.fnode.path.cmp(&r.fnode.path));

		let mut tvp = TreeViewPainter::new(self, &mut lofnodes);
		tvp.place_fnodes();
		tvp.paint_div_lines();
		tvp.action
	}
}

fn create_fnode<'node>(wp: &mut WidgetPlacer, fnode: FNode<'node>, highlighted: bool) -> LaidOutFNode<'node> {
	let label_text = if highlighted {
		RichText::new(&fnode.node.name).color(Color32::YELLOW)
	} else {
		RichText::new(&fnode.node.name)
	};
	let ((checkbox, text, add_button, del_button), rect1) = wp.right_to_left(|wp| {
		let del_button = wp.create(Button::new(" 🗑 "));
		let add_button = wp.create(Button::new(" + "));
		let ((checkbox, text), _) = wp.left_to_right(|wp| {
			wp.add_space(INDENT_SPACING * fnode.depth as f32);
			(
				wp.create(Checkbox::without_text(fnode.selected)),
				wp.create(Label::new(label_text).wrap(true).sense(Sense::click())),
			)
		});
		(checkbox, text, add_button, del_button)
	});

	if fnode.node.session.is_none() && fnode.node.due_date.is_none() {
		return LaidOutFNode {
			fnode,
			checkbox,
			text,
			add_button,
			del_button,
			session_label: None,
			due_date_label: None,
			height1: rect1.height(),
			height2: 0.,
		};
	}
	let ((session_label, due_date_label), rect2) = wp.right_to_left(|wp| {
		let due_date_label = fnode.node.due_date.map(|due_date| {
			let due_date = format!("{}", Displayable(Some(due_date)));
			wp.create(Label::new(due_date))
		});
		let (session_label, _) = wp.left_to_right(|wp| {
			wp.add_space(INDENT_SPACING * fnode.depth as f32);
			fnode.node.session.map(|session| {
				let session = format!("{}", Displayable(Some(session)));
				wp.create(Label::new(session).wrap(true))
			})
		});
		(session_label, due_date_label)
	});
	LaidOutFNode { fnode, checkbox, text, add_button, del_button, session_label, due_date_label, height1: rect1.height(), height2: rect2.height() }
}

struct TreeViewPainter<'ui, 'lofnodes, 'node> {
	ui: &'ui mut Ui,
	lofnodes: &'lofnodes mut[LaidOutFNode<'node>],
	maxy: f32,
	color_map: HashMap<u64, Color32>,
	action: Action,
}

impl<'ui, 'lofnodes, 'node> TreeViewPainter<'ui, 'lofnodes, 'node> {
	fn new(ui: &'ui mut Ui, lofnodes: &'lofnodes mut[LaidOutFNode<'node>]) -> Self {
		let mut maxy = ui.next_widget_position().y;
		let mut color_map = HashMap::new();
		for lofnode in lofnodes.iter() {
			if let Some(color) = color_map.get_mut(&lofnode.fnode.node.id) {
				if *color == Color32::WHITE {
					let mut hasher = DefaultHasher::new();
					hasher.write_u64(lofnode.fnode.node.id);
					let hash = hasher.finish().to_le_bytes();
					*color = Color32::from_rgb(hash[0], hash[1], hash[2]);
				}
			} else {
				color_map.insert(lofnode.fnode.node.id, Color32::WHITE);
			}

			maxy += lofnode.height(ui.spacing().item_spacing.y);
		}
		TreeViewPainter { ui, lofnodes, maxy, color_map, action: Action::None }
	}

	fn place_fnodes(&mut self) {
		let spacing = self.ui.spacing().item_spacing.y;
		let mut h = self.ui.next_widget_position().y;
		for lofnode in self.lofnodes.iter_mut() {
			lofnode.checkbox.reposition(h + (lofnode.height(spacing) - spacing) / 2.0);
			let checkbox_response = lofnode.checkbox.interact(self.ui);
			self.ui.paint(&lofnode.checkbox, &checkbox_response);

			if checkbox_response.clicked() {
				self.action = Action::Toggle(lofnode.fnode.pid, lofnode.fnode.node.id);
			}

			lofnode.text.reposition(h);
			let label_response = lofnode.text.interact(self.ui);
			self.ui.paint(&lofnode.text, &label_response);

			if lofnode.fnode.pid != lofnode.fnode.node.id && label_response.clicked() {
				self.action = Action::MoveInto(lofnode.fnode.pid, lofnode.fnode.node.id);
			}

			lofnode.add_button.reposition(h + (lofnode.height(spacing) - spacing) / 2.0);
			let add_response = lofnode.add_button.interact(self.ui);
			self.ui.paint(&lofnode.add_button, &add_response);

			if add_response.clicked() {
				self.action = Action::Add(lofnode.fnode.pid, lofnode.fnode.node.id);
			}

			if lofnode.fnode.pid != lofnode.fnode.node.id {
				lofnode.del_button.reposition(h + (lofnode.height(spacing) - spacing) / 2.0);
				let del_response = lofnode.del_button.interact(self.ui);
				self.ui.paint(&lofnode.del_button, &del_response);

				if del_response.clicked() {
					self.action = Action::Delete(lofnode.fnode.pid, lofnode.fnode.node.id);
				}
			}

			h += lofnode.height1 + self.ui.spacing().item_spacing.y;

			if let Some(session_label) = &mut lofnode.session_label {
				session_label.reposition(h);
				let label_response = session_label.interact(self.ui);
				self.ui.paint(session_label, &label_response);
			}

			if let Some(due_date_label) = &mut lofnode.due_date_label {
				due_date_label.reposition(h);
				let label_response = due_date_label.interact(self.ui);
				self.ui.paint(due_date_label, &label_response);
			}

			if lofnode.session_label.is_some() || lofnode.due_date_label.is_some() {
				h += lofnode.height2 + self.ui.spacing().item_spacing.y;
			}
		}
	}

	fn paint_div_lines(&mut self) {
		let mut line_pos = Vec::new();
		let mut h = self.maxy;
		for lofnode in self.lofnodes.iter().skip(1).rev() {
			h -= lofnode.height(self.ui.spacing().item_spacing.y);
			match line_pos.last() {
				Some(&last) if lofnode.fnode.depth < last => {
					line_pos.pop();
					if line_pos.last() == Some(&lofnode.fnode.depth) {
						self.paint_div_line(lofnode, h, &line_pos, self.ui.spacing().item_spacing.y);
					} else {
						line_pos.push(lofnode.fnode.depth);
						self.paint_div_line(lofnode, h, &line_pos, self.ui.spacing().item_spacing.y);
					}
				}
				Some(&last) if lofnode.fnode.depth == last => {
					self.paint_div_line(lofnode, h, &line_pos, self.ui.spacing().item_spacing.y);
				}
				_ => {
					line_pos.push(lofnode.fnode.depth);
					self.paint_div_line(lofnode, h, &line_pos, self.ui.spacing().item_spacing.y);
				}
			}
		}
	}

	fn paint_div_line(
		&self,
		lofnode: &LaidOutFNode,
		h: f32,
		line_pos: &[usize],
		spacing: f32,
	) {
		let x = self.ui.next_widget_position().x + 7.;
		let color = *self.color_map.get(&lofnode.fnode.node.id).unwrap();
		for pos in &line_pos[..line_pos.len() - 1] {
			self.ui.painter().line_segment([
				Pos2::new(x + (pos - 1) as f32 * INDENT_SPACING, h),
				Pos2::new(x + (pos - 1) as f32 * INDENT_SPACING, h + lofnode.height(spacing)),
			], self.ui.style().noninteractive().fg_stroke);
		}
		let endpos = line_pos[line_pos.len() - 1];
		if lofnode.fnode.priority.is_least() {
			self.ui.painter().line_segment([
				Pos2::new(x + (endpos - 1) as f32 * INDENT_SPACING, h),
				Pos2::new(x + (endpos - 1) as f32 * INDENT_SPACING, h + (lofnode.height(spacing) - spacing) / 2.0),
			], self.ui.style().noninteractive().fg_stroke);
		} else {
			self.ui.painter().line_segment([
				Pos2::new(x + (endpos - 1) as f32 * INDENT_SPACING, h),
				Pos2::new(x + (endpos - 1) as f32 * INDENT_SPACING, h + lofnode.height(spacing)),
			], self.ui.style().noninteractive().fg_stroke);
		}
		self.ui.painter().line_segment([
			Pos2::new(x + (endpos - 1) as f32 * INDENT_SPACING, h + (lofnode.height(spacing) - spacing) / 2.0),
			Pos2::new(x + (endpos as f32 - 0.5) * INDENT_SPACING, h + (lofnode.height(spacing) - spacing) / 2.0),
		], self.ui.style().noninteractive().fg_stroke);
	}
}

fn color_from_prio(prio: &Priority) -> Color32 {
	color_from_hsv((prio.det * 120) as f64 / prio.total as f64, 1.0, 1.0)
}

fn color_from_hsv(hue: f64, saturation: f64, value: f64) -> Color32 {
	let c = value * saturation;
	let h = hue / 60.0;
	let x = c * (1.0 - (h % 2.0 - 1.0).abs());
	let m = value - c;

	let (red, green, blue) = if h >= 0.0 && h < 1.0 {
		(c, x, 0.0)
	} else if h >= 1.0 && h < 2.0 {
		(x, c, 0.0)
	} else if h >= 2.0 && h < 3.0 {
		(0.0, c, x)
	} else if h >= 3.0 && h < 4.0 {
		(0.0, x, c)
	} else if h >= 4.0 && h < 5.0 {
		(x, 0.0, c)
	} else {
		(c, 0.0, x)
	};

	Color32::from_rgb(
		((red + m) * 255.0) as u8,
		((green + m) * 255.0) as u8,
		((blue + m) * 255.0) as u8,
	)
}

struct LaidOutFNode<'node> {
	fnode: FNode<'node>,
	checkbox: LaidOutCheckbox,
	text: LaidOutLabel,
	add_button: LaidOutButton,
	del_button: LaidOutButton,
	session_label: Option<LaidOutLabel>,
	due_date_label: Option<LaidOutLabel>,
	height1: f32,
	height2: f32,
}

impl LaidOutFNode<'_> {
	fn height(&self, spacing: f32) -> f32 {
		let mut h = self.height1 + spacing;
		if self.session_label.is_some() || self.due_date_label.is_some() {
			h += self.height2 + spacing;
		}
		h
	}
}

struct FNode<'node> {
	node: &'node Node,
	path: Vec<usize>,
	pid: u64,
	depth: usize,
	selected: bool,
	priority: Priority,
}

struct Priority {
	det: u64,
	total: u64,
}

impl Priority {
	pub fn is_least(&self) -> bool {
		self.det + 1 == self.total
	}
}

struct FChildIter<'node> {
	iter: IntoIter<FNode<'node>>,
}

impl<'node> FChildIter<'node> {
	fn new(fnode: &FNode, tree: &'node Tree) -> Self {
		let mut children = Vec::new();
		for node in tree.children(fnode.node.id) {
			children.push(FNode {
				node,
				path: fnode.path.clone(),
				pid: fnode.node.id,
				depth: fnode.path.len(),
				selected: tree.is_selected(fnode.node.id, node.id),
				priority: Priority { det: 0, total: 0 },
			});
		}
		for i in 0..children.len() {
			children[i].priority = Priority {
				det: i as u64,
				total: children.len() as u64,
			}
		}
		children.sort_by(|l, r| l.priority.det.cmp(&r.priority.det));
		FChildIter { iter: children.into_iter() }
	}
}
