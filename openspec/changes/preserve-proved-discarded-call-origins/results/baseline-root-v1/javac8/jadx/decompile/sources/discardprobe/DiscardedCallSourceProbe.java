package discardprobe;

import java.util.ArrayList;

/* JADX INFO: loaded from: DiscardedCallSourceProbe.class */
public final class DiscardedCallSourceProbe {
    private DiscardedCallSourceProbe() {
    }

    public static String give(boolean z) {
        if (z) {
            throw new IllegalStateException("give-failed");
        }
        return "given";
    }

    public static void discardStatic(boolean z) {
        give(z);
    }

    public static String discardAppend(String str) {
        return str;
    }

    public static String discardListAdd(String str) {
        ArrayList arrayList = new ArrayList();
        arrayList.add(str);
        return (String) arrayList.get(0);
    }

    public static String consumeReturn() {
        return give(false);
    }

    public static String deferToLocal() {
        return give(false);
    }
}
