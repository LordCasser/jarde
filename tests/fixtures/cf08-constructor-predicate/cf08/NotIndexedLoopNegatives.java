package cf08;

import java.io.File;

public final class NotIndexedLoopNegatives {
	public static File constructorAliasStore(File[] files) {
		File file;
		if (files != null) {
			int length = files.length;
			if (length == 0) {
				file = null;
			} else {
				int i = 0;
				while (true) {
					if (i >= length) {
						File extra = new File("h");
						file = extra;
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

	public static File constructorExtraConsumer(File[] files) {
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
						file.getPath();
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

	public static File constructorEffectArgument(File[] files) {
		File file;
		if (files != null) {
			int length = files.length;
			if (length == 0) {
				file = null;
			} else {
				int i = 0;
				while (true) {
					if (i >= length) {
						file = new File(new String("h"));
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

	public static File predicateExtraCall(File[] files) {
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
					if (file.getName().trim().equals("f")) {
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

	public static File predicateOtherReceiver(File[] files) {
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
					if (files[i].getName().equals("f")) {
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

	public static File predicateExtraEffect(File[] files) {
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
					file.deleteOnExit();
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

	public static File extraInnerExit(File[] files) {
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
					if (i == -1) { file = null; break; }
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

	public static File innerJoinEffect(File[] files) {
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
			String.valueOf(file);
		} else {
			file = null;
		}
		if (file != null) {
			file.deleteOnExit();
		}
		return file;
	}

	public static File outerNullObject(File[] files) {
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
			file = new File("x");
		}
		if (file != null) {
			file.deleteOnExit();
		}
		return file;
	}
}
