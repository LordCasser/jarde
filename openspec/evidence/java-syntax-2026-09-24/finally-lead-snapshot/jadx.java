package defpackage;

/* JADX INFO: loaded from: FinallyLeadSnapshot.class */
public class FinallyLeadSnapshot {
    public static int value;

    public static int run() {
        value = 41;
        try {
            int i = 99;
            return value;
        } finally {
            value = 99;
        }
    }
}
