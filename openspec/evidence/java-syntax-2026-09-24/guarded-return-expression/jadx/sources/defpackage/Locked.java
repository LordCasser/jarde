package defpackage;

/* JADX INFO: loaded from: Locked.class */
public class Locked {
    int n;

    int locked() {
        int i;
        synchronized (this) {
            i = this.n;
        }
        return i;
    }
}
