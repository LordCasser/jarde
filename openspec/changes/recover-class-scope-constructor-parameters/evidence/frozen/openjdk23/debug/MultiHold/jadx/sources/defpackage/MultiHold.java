package defpackage;

import java.lang.CharSequence;

/* JADX INFO: loaded from: MultiHold.jar:MultiHold.class */
public class MultiHold<T, U extends CharSequence> {
    public T first;
    public U second;

    public MultiHold(T first, U second) {
        this.first = first;
        this.second = second;
    }
}
