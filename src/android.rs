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

use jni::JavaVM;
use jni::errors::Result;
use jni::objects::JObject;
use jni::sys::jobject;
use winit::platform::android::activity::AndroidApp;

pub struct JniWrapper {
	vm: JavaVM,
	activity: JObject<'static>,
}

impl JniWrapper {
	pub fn new(app: &AndroidApp) -> Result<Self> {
		Ok(Self {
			vm: unsafe { JavaVM::from_raw(app.vm_as_ptr() as _) }?,
			activity: unsafe { JObject::from_raw(app.activity_as_ptr() as jobject) },
		})
	}

	pub fn import(&self) -> Result<()> {
		let mut env = self.vm.attach_current_thread()?;
		env.call_method(&self.activity, "importStore", "()V", &[])?;
		Ok(())
	}

	pub fn export(&self) -> Result<()> {
		let mut env = self.vm.attach_current_thread()?;
		env.call_method(&self.activity, "exportStore", "()V", &[])?;
		Ok(())
	}
}
