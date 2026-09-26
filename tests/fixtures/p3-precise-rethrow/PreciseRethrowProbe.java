import java.io.IOException;
import java.text.ParseException;

public final class PreciseRethrowProbe {
    static String trace = "";

    static void log(Exception exception) {
        trace += exception.getClass().getName() + "/" + exception.getMessage() + "|";
    }

    static int precise(int mode) throws ParseException, IOException {
        try {
            if (mode == 1) throw new ParseException("parse", 3);
            if (mode == 2) throw new IOException("io");
            return 23;
        } catch (Exception e) {
            log(e);
            throw e;
        }
    }
}
