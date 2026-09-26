package defpackage;

import java.io.IOException;
import java.text.ParseException;

/* JADX INFO: loaded from: PreciseRethrowBoundaryProbe.class */
public final class PreciseRethrowBoundaryProbe {
    static String trace = "";

    static void log(Exception exception) {
        trace += exception.getClass().getName() + "/" + exception.getMessage() + "|";
    }

    static void log(String text) {
        trace += text + "|";
    }

    static int anyFinally(int mode) {
        if (mode != 0) {
            try {
                throw new IllegalStateException("finally-body");
            } catch (Throwable th) {
                log("finally");
                throw th;
            }
        }
        log("finally");
        return 17;
    }

    static int multiCatch(int mode) throws Exception {
        try {
            if (mode == 1) {
                throw new ParseException("multi-parse", 5);
            }
            if (mode == 2) {
                throw new IOException("multi-io");
            }
            return 29;
        } catch (IOException | ParseException e) {
            log(e);
            throw e;
        }
    }

    static int changedValue(int mode) throws Exception {
        if (mode == 0) {
            return 31;
        }
        try {
            throw new IOException("input");
        } catch (Exception e) {
            log(e);
            throw new IllegalStateException("replacement");
        }
    }
}
