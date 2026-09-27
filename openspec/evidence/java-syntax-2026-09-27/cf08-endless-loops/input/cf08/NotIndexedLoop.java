package cf08;

import java.io.File;

public final class NotIndexedLoop {
	public File test(File[] files) {
		File file;
		if (files != null) {
			int length = files.length;
			if (length == 0) {
				file = null;
			} else {
				int i = 0;
				while (true) {
					if (i >= length) {
						file = new File("h");
						break;
					}
					file = files[i];
					if (file.getName().equals("f")) {
						break;
					}
					i++;
				}
			}
		} else {
			file = null;
		}
		if (file != null) {
			file.deleteOnExit();
		}
		return file;
	}
}
