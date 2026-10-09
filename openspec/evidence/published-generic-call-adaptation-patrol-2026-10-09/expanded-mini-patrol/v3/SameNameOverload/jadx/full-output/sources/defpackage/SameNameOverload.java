package defpackage;

/* JADX INFO: loaded from: SameNameOverload.jar:SameNameOverload.class */
public class SameNameOverload<T> {
    public String selected;

    public void pick(T x) {
        this.selected = "generic";
    }

    public void pick(String x) {
        this.selected = "string";
    }

    public void relay(T x) {
        pick(x);
    }
}
