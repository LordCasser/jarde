package jadx.tests.integration.trycatch;

import java.io.File;
import java.io.IOException;
import java.io.OutputStream;

/* JADX INFO: loaded from: Control.class */
public class Control {
    public static void run(OutputStream outputStream, File file) throws IOException {
        try {
            outputStream.write(1);
        } finally {
            try {
                outputStream.close();
                file.delete();
            } catch (IOException e) {
            }
        }
    }
}
