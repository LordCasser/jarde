package defpackage;

import java.lang.Comparable;
import java.lang.Number;

/* JADX INFO: loaded from: BoundRawParam.jar:BoundRawParam.class */
public class BoundRawParam<T extends Number & Comparable<T>> {
    public T value;

    /* JADX WARN: Multi-variable type inference failed */
    public static void put(BoundRawParam boundRawParam, Number number) {
        boundRawParam.value = number;
    }
}
