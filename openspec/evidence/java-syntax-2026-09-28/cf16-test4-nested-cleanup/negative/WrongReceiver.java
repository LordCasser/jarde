package jadx.tests.integration.trycatch;

import java.io.File;
import java.io.IOException;
import java.io.OutputStream;

/** A separate verifier-valid neighbor: the exceptional copy closes another receiver. */
public class WrongReceiver {
    public static void run(OutputStream outputStream, OutputStream other, File file)
            throws IOException {
        try {
            outputStream.write(1);
        } catch (Throwable primary) {
            try {
                other.close();
                file.delete();
            } catch (IOException ignored) {
            }
            throw primary;
        }
        try {
            outputStream.close();
            file.delete();
        } catch (IOException ignored) {
        }
    }
}
