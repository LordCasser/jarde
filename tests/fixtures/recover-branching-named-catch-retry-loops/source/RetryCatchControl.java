package recover.loop2;

import java.util.ArrayList;
import java.util.List;

/** Reachable, deterministic control for a conditional handler with a retry leaf. */
public final class RetryCatchControl {
    static final List<String> events = new ArrayList<>();
    static int attempts;
    static int catches;
    static boolean failFirst;
    static boolean predicateValue;
    static boolean failRetryEffect;
    static Exception saved;

    private RetryCatchControl() {}

    static void reset(boolean shouldFailFirst, boolean shouldThrowWhenTrue,
            boolean shouldFailRetryEffect) {
        events.clear();
        attempts = 0;
        catches = 0;
        failFirst = shouldFailFirst;
        predicateValue = shouldThrowWhenTrue;
        failRetryEffect = shouldFailRetryEffect;
        saved = null;
    }

    static void trace(String event) {
        events.add(event);
    }

    public static void run() throws Exception {
        while (true) {
            try {
                funcB();
                break;
            } catch (Exception ex) {
                if (funcC(ex)) throw ex;
                retryEffect();
            }
        }
        trace("return");
    }

    public static void runInverted() throws Exception {
        while (true) {
            try {
                funcB();
                break;
            } catch (Exception ex) {
                if (!funcC(ex)) throw ex;
                retryEffect();
            }
        }
        trace("return");
    }

    private static void funcB() throws Exception {
        attempts++;
        trace("funcB:" + attempts);
        if (failFirst && attempts == 1) {
            saved = new Exception("B-once");
            throw saved;
        }
    }

    private static boolean funcC(Exception ex) {
        catches++;
        trace("funcC:" + ex.getMessage() + ":" + predicateValue);
        return predicateValue;
    }

    private static void retryEffect() {
        trace("clear");
        trace("retry-effect");
        if (failRetryEffect) throw new IllegalStateException("retry-effect-failed");
    }
}
