package defpackage;

import java.util.ArrayList;
import java.util.List;

/* JADX INFO: loaded from: RawReceiver.jar:RawReceiver.class */
public class RawReceiver<T> {
    public List box = new ArrayList();

    public T relay(T t) {
        this.box.add(t);
        return (T) this.box.get(0);
    }
}
