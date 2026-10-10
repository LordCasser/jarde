package discardprobe;

import java.util.ArrayList;
import java.util.List;

/** Small Java 8 source-map and whole-class replay fixture for invoke-result POPs. */
public final class DiscardedCallSourceProbe {
    private DiscardedCallSourceProbe() {}

    public static String give(boolean fail) {
        if (fail) {
            throw new IllegalStateException("give-failed");
        }
        return "given";
    }

    public static void discardStatic(boolean fail) {
        give(fail);
    }

    public static String discardAppend(String value) {
        StringBuilder builder = new StringBuilder();
        builder.append(value);
        return builder.toString();
    }

    public static String discardListAdd(String value) {
        List<String> values = new ArrayList<String>();
        values.add(value);
        return values.get(0);
    }

    public static String consumeReturn() {
        return give(false);
    }

    public static String deferToLocal() {
        String value = give(false);
        return value;
    }
}
