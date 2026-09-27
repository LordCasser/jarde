package cf08;

import java.io.File;

/* JADX INFO: loaded from: input.jar:cf08/NotIndexedLoop.class */
public final class NotIndexedLoop {
    public File test(File[] fileArr) {
        File file;
        int length;
        if (fileArr == null || (length = fileArr.length) == 0) {
            file = null;
        } else {
            int i = 0;
            while (true) {
                if (i >= length) {
                    file = new File("h");
                    break;
                }
                file = fileArr[i];
                if (file.getName().equals("f")) {
                    break;
                }
                i++;
            }
        }
        if (file != null) {
            file.deleteOnExit();
        }
        return file;
    }
}
