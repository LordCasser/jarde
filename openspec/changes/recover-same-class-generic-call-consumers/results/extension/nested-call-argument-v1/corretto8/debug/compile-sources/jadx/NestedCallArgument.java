package defpackage;

/* JADX INFO: loaded from: NestedCallArgument.input.jar:NestedCallArgument.class */
public class NestedCallArgument<T> {
    public T first(T x) {
        return x;
    }

    public T second(T x) {
        return x;
    }

    public T relay(T t) {
        return second(first(t));
    }
}
