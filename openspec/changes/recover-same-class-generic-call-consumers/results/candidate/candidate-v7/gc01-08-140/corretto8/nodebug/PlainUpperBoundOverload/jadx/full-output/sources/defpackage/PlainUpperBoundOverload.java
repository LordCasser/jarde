package defpackage;

import java.lang.Number;

/* JADX INFO: loaded from: PlainUpperBoundOverload.jar:PlainUpperBoundOverload.class */
public class PlainUpperBoundOverload<T extends Number> {
    public String selected;

    public void pick(Number number) {
        this.selected = "number";
    }

    public void pick(Object obj) {
        this.selected = "object";
    }

    public void relay(T t) {
        pick((Number) t);
    }
}
