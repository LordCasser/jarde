package defpackage;

/* JADX INFO: loaded from: NestedCallArgument.input.jar:NestedCallArgument.class */
public class NestedCallArgument<T> {
    public T first(T t) {
        return t;
    }

    public T second(T t) {
        return t;
    }

    public T relay(T t) {
        return second(first(t));
    }
}
