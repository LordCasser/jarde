package defpackage;

/* JADX INFO: loaded from: SameNameOverload.jar:SameNameOverload.class */
public class SameNameOverload<T> {
    public String selected;

    public void pick(T t) {
        this.selected = "generic";
    }

    public void pick(String str) {
        this.selected = "string";
    }

    public void relay(T t) {
        pick(t);
    }
}
