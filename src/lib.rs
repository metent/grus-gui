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

#[cfg(target_os = "android")]
mod android;
mod app;
mod node;
mod ftree;
mod vboard;

use eframe::NativeOptions;
use std::path::Path;
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
#[cfg(target_os = "android")]
use android::JniWrapper;
use app::Grus;

pub fn _main<P: AsRef<Path>>(
	options: NativeOptions,
	data_path: P,
	#[cfg(target_os = "android")] jniwr: JniWrapper
) {
	let app = match Grus::new(data_path, 2, #[cfg(target_os = "android")] jniwr) {
		Ok(app) => app,
		Err(err) => { eprintln!("Failed to open store: {}", err); return; }
	};
	#[cfg(target_os = "android")]
	let scale = 3.5;
	#[cfg(not(target_os = "android"))]
	let scale = 1.5;
	if let Err(err) = eframe::run_native("Grus", options, Box::new(move |cc| Box::new(app.with_scale(cc, scale).with_fonts(cc)))) {
		eprintln!("Failed to start GUI: {}", err);
	}
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
	let Some(mut data_dir) = app.internal_data_path() else {
		eprintln!("Failed to fetch data directory.");
		return;
	};
	data_dir.push("tasks");
	let jniwr = match JniWrapper::new(&app) {
		Ok(jniwr) => jniwr,
		Err(err) => { eprintln!("Failed to create JNI wrapper: {}", err); return; },
	};
	_main(NativeOptions {
		renderer: eframe::Renderer::Wgpu,
		event_loop_builder: Some(Box::new(move |b| _ = b.with_android_app(app))),
		..NativeOptions::default()
	}, data_dir, jniwr);
}
