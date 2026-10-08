package defpackage;

/* JADX INFO: loaded from: NoCheck.class */
public class NoCheck {
    public static java.lang.Thread make(java.lang.Thread thread) {
        return new java.lang.Thread(thread::start);
    }
}
