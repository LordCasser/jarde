package defpackage;

import java.util.ArrayList;

/* JADX INFO: loaded from: MutatedReceiver.jar:MutatedReceiver.class */
public class MutatedReceiver<T> {
    public T relay(T t) {
        new ArrayList();
        ArrayList arrayList = new ArrayList();
        arrayList.add(t);
        return (T) arrayList.get(0);
    }

    /* JADX WARN: Multi-variable type inference failed */
    public static void main(String[] a) {
        Object m = new Object();
        System.out.println("behavior.marker=" + (new MutatedReceiver().relay(m) == m));
    }
}
