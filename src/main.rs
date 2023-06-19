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

use std::fs;
use std::io::{self, ErrorKind};
use eframe::{NativeOptions, Renderer};
use grus_gui::_main;

fn main() -> io::Result<()> {
	let mut pathbuf = dirs::data_dir().ok_or_else(|| io::Error::new(
		ErrorKind::NotFound,
		"No data directory found.",
	))?;
	pathbuf.push("grus");
	fs::create_dir_all(&pathbuf)?;
	pathbuf.push("tasks");

	#[cfg(not(target_os = "android"))]
	_main(NativeOptions {
		renderer: Renderer::Wgpu,
		..NativeOptions::default()
	}, pathbuf);
	Ok(())
}
