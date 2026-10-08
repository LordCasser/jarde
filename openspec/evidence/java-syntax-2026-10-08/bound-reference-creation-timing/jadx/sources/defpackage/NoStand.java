package defpackage;

/* JADX INFO: loaded from: NoStand.class */
public class NoStand {
    public static java.lang.Runnable make(java.lang.Thread thread) {
        return thread::start;
    }
}
