import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public class Combo {
    static int read(byte[] data) throws IOException {
        try (InputStream in = new ByteArrayInputStream(data)) {
            return in.read();
        } catch (IOException e) {
            return -1;
        }
    }
}
