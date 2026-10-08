package defpackage;

import java.lang.Comparable;
import java.util.Collections;

/* JADX INFO: loaded from: SCGA.jar:SCGA.class */
public class SCGA<T extends Comparable<T>> {
    public T v;

    public void put(T t) {
        Collections.singletonList(t);
        this.v = t;
    }

    public static void main(String[] strArr) {
        SCGA scga = new SCGA();
        scga.put("same-class");
        System.out.println("same-class:" + ((String) scga.v));
    }
}
