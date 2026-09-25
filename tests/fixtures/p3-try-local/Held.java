import java.io.IOException;
import java.io.Reader;

public class Held {
    static int use(Reader r) throws IOException {
        try (r) {
            return r.read();
        }
    }
}
