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

package com.github.grusgui;

import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.IOException;
import android.app.NativeActivity;
import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.database.Cursor;
import android.net.Uri;
import android.os.ParcelFileDescriptor;
import android.provider.DocumentsContract;

public class MainActivity extends NativeActivity {
	static {
		System.loadLibrary("grus_gui");
	}

	private static final int IMPORT_REQUEST = 0;
	private static final int EXPORT_REQUEST = 1;
	private Uri baseDocumentTreeUri;

	@Override
	public void onActivityResult(int requestCode, int resultCode, Intent data) {
		baseDocumentTreeUri = data.getData();
		if (baseDocumentTreeUri != null) switch (requestCode) {
			case IMPORT_REQUEST: importStore(); break;
			case EXPORT_REQUEST: exportStore(); break;
			default: return;
		} else return;
		final int takeFlags = (Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION);

		getContentResolver().takePersistableUriPermission(baseDocumentTreeUri, takeFlags);
		SharedPreferences preferences = getSharedPreferences("com.github.grusgui", Context.MODE_PRIVATE);
		preferences.edit().putString("filestorageuri", baseDocumentTreeUri.toString()).apply();
	}

	void getBaseDocumentTreeUri(int requestCode) {
		SharedPreferences preferences = getSharedPreferences("com.github.grusgui", Context.MODE_PRIVATE);
		String uriString = preferences.getString("filestorageuri", null);
		if (uriString == null) {
			Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT_TREE);
			startActivityForResult(intent, requestCode);
		} else {
			baseDocumentTreeUri = Uri.parse(uriString);
		}
	}

	void importStore() {
		if (baseDocumentTreeUri == null) getBaseDocumentTreeUri(IMPORT_REQUEST);
		if (baseDocumentTreeUri == null) return;
		try {
			Uri storeUri = getStoreUri();
			ParcelFileDescriptor pfd = getContentResolver().openFileDescriptor(storeUri, "r");
			FileInputStream in = new FileInputStream(pfd.getFileDescriptor());
			FileOutputStream out = new FileOutputStream(getApplicationInfo().dataDir + "/files/tasks");
			copyStream(in, out);

		} catch (FileNotFoundException e) {
			e.printStackTrace();
		}
	}

	void exportStore() {
		if (baseDocumentTreeUri == null) getBaseDocumentTreeUri(EXPORT_REQUEST);
		if (baseDocumentTreeUri == null) return;
		try {
			Uri storeUri = getStoreUri();
			ParcelFileDescriptor pfd = getContentResolver().openFileDescriptor(storeUri, "w");
			FileInputStream in = new FileInputStream(getApplicationInfo().dataDir + "/files/tasks");
			FileOutputStream out = new FileOutputStream(pfd.getFileDescriptor());
			copyStream(in, out);

		} catch (FileNotFoundException e) {
			e.printStackTrace();
		}
	}

	Uri getStoreUri() {
		String baseDocumentId = DocumentsContract.getTreeDocumentId(baseDocumentTreeUri);
		Uri uri = DocumentsContract.buildDocumentUriUsingTree(baseDocumentTreeUri, baseDocumentId);
		Uri childrenUri = DocumentsContract.buildChildDocumentsUriUsingTree(uri,
			DocumentsContract.getDocumentId(uri));
		Cursor c = getContentResolver().query(childrenUri, new String[] {
			DocumentsContract.Document.COLUMN_DOCUMENT_ID,
			DocumentsContract.Document.COLUMN_DISPLAY_NAME,
		}, null, null, null);

		Uri storeUri = null;
		while (c.moveToNext()) {
			String docId = c.getString(0);
			String name = c.getString(1);
			if (name.equals("tasks")) {
				storeUri = DocumentsContract.buildDocumentUriUsingTree(uri, docId);
				break;
			}
		}
		c.close();

		if (storeUri == null) try {
			storeUri = DocumentsContract.createDocument(getContentResolver(), uri, "text/*", "tasks");
		} catch (FileNotFoundException e) {
			e.printStackTrace();
		}
		return storeUri;
	}

	void copyStream(FileInputStream in, FileOutputStream out) {
		byte[] buffer = new byte[1024];
		int read;
		try {
			while ((read = in.read(buffer)) != -1) {
				out.write(buffer, 0, read);
			}
			in.close();
			out.flush();
			out.close();
		} catch (IOException e) {
			e.printStackTrace();
		}
	}
}
