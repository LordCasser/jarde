package defpackage;

import java.lang.Number;

/* JADX INFO: loaded from: PlainUpperBoundOverload.jar:PlainUpperBoundOverload.class */
public class PlainUpperBoundOverload<T extends Number> {
    public String selected;

    public void pick(Number x) {
        this.selected = "number";
    }

    public void pick(Object x) {
        this.selected = "object";
    }

    public void relay(T x) {
        pick((Number) x);
    }
}
