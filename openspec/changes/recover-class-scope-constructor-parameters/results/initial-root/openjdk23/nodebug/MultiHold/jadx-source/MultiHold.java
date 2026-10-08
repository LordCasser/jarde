import java.lang.CharSequence;

/* JADX INFO: loaded from: MultiHold.jar:MultiHold.class */
public class MultiHold<T, U extends CharSequence> {
    public T first;
    public U second;

    public MultiHold(T t, U u) {
        this.first = t;
        this.second = u;
    }
}
