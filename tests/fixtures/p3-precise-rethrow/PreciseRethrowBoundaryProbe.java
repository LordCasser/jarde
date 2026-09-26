import java.io.IOException;
import java.text.ParseException;

public final class PreciseRethrowBoundaryProbe {
    static String trace = "";

    static void log(Exception exception) {
        trace += exception.getClass().getName() + "/" + exception.getMessage() + "|";
    }

    static void log(String text) {
        trace += text + "|";
    }

    static int anyFinally(int mode) {
        try {
            if (mode != 0) throw new IllegalStateException("finally-body");
            return 17;
        } finally {
            log("finally");
        }
    }

    static int multiCatch(int mode) throws ParseException, IOException {
        try {
            if (mode == 1) throw new ParseException("multi-parse", 5);
            if (mode == 2) throw new IOException("multi-io");
            return 29;
        } catch (ParseException | IOException e) {
            log(e);
            throw e;
        }
    }

    static int changedValue(int mode) throws Exception {
        try {
            if (mode != 0) throw new IOException("input");
            return 31;
        } catch (Exception e) {
            log(e);
            throw new IllegalStateException("replacement");
        }
    }
}
