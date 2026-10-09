package defpackage;

import java.util.ArrayList;

/* JADX INFO: loaded from: RawAllocationVariable.jar:RawAllocationVariable.class */
public class RawAllocationVariable<T> {
    public T v;

    public void put() {
        this.v = (T) new ArrayList();
    }
}
