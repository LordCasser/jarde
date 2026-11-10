package flow;

public class HeaderUnclosed<T> extends GenericParent<T> {
    public @TypeTag T value;

    public T echo(T value) { return value; }

    public HeaderUnclosed() {
        super();
    }
}
