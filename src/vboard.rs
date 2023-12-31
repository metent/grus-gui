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

use egui::{Button, RichText, Ui, Vec2};

pub trait VBoard {
	fn vboard(&mut self) -> Option<Key>;
	fn caps_vboard(&mut self) -> Option<Key>;
}

impl VBoard for Ui {
	fn vboard(&mut self) -> Option<Key> {
		let key0 = self.row(None, &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "="], Some((Key::Backspace, 40.0)));
		let key1 = self.row(None, &["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"], None);
		let key2 = self.row(Some((Key::CapsLock, 40.0)), &["a", "s", "d", "f", "g", "h", "j", "k", "l", ";", "'"], None);
		let key3 = self.row(None, &["z", "x", "c", "v", "b", "n", "m", ",", ".", "/"], None);
		let key4 = self.row(None, &[" "], Some((Key::Enter, 30.0)));
		key0.or(key1).or(key2).or(key3).or(key4)
	}

	fn caps_vboard(&mut self) -> Option<Key> {
		let key0 = self.row(None, &["!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "_", "+"], Some((Key::Backspace, 40.0)));
		let key1 = self.row(None, &["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"], None);
		let key2 = self.row(Some((Key::CapsLock, 40.0)), &["A", "S", "D", "F", "G", "H", "J", "K", "L", ";", "'"], None);
		let key3 = self.row(None, &["Z", "X", "C", "V", "B", "N", "M", ",", ".", "/"], None);
		let key4 = self.row(None, &[" "], Some((Key::Enter, 30.0)));
		key0.or(key1).or(key2).or(key3).or(key4)
	}
}

trait VBoardExt {
	fn row(&mut self, start: Option<(Key, f32)>, keys: &[&str], end: Option<(Key, f32)>) -> Option<Key>;
}

impl VBoardExt for Ui {
	fn row(&mut self, start: Option<(Key, f32)>, keys: &[&str], end: Option<(Key, f32)>) -> Option<Key> {
		let (start_key, start_width) = match start {
			Some((start_key, start_width)) => (Some(start_key), start_width + 5.0),
			None => (None, 0.),
		};
		let (end_key, end_width) = match end {
			Some((end_key, end_width)) => (Some(end_key), end_width + 5.0),
			None => (None, 0.),
		};
		let width = self.available_width() - start_width - end_width;
		let mut pressed = None;
		self.horizontal(|ui| {
			ui.spacing_mut().item_spacing.x = 5.0;
			if let Some(start_key) = start_key {
				let size = Vec2::new(start_width - 5.0, ui.available_height());
				let (_, rect) = ui.allocate_space(size);
				let btn_txt = match start_key {
					Key::CapsLock => "Caps",
					_ => unimplemented!(),
				};
				let btn = Button::new(RichText::new(btn_txt).size(10.0)).min_size(size);
				if ui.put(rect, btn).clicked() {
					pressed = Some(start_key);
				}
			}
			for key in keys {
				let size = Vec2::new(width / keys.len() as f32 - 5.0, ui.available_height());
				let (_, rect) = ui.allocate_space(size);
				let btn = Button::new(RichText::new(*key).size(10.0)).min_size(size);
				if ui.put(rect, btn).clicked() {
					pressed = key.chars().nth(0).map(|c| Key::Char(c));
				}
			}
			if let Some(end_key) = end_key {
				let size = Vec2::new(end_width - 5.0, ui.available_height());
				let (_, rect) = ui.allocate_space(size);
				let btn_txt = match end_key {
					Key::Enter => ">",
					Key::Backspace => "<-",
					_ => unimplemented!(),
				};
				let btn = Button::new(RichText::new(btn_txt).size(10.0)).min_size(size);
				if ui.put(rect, btn).clicked() {
					pressed = Some(end_key);
				}
			}
		});
		pressed
	}
}

pub enum Key {
	Char(char),
	Enter,
	Backspace,
	CapsLock,
}
