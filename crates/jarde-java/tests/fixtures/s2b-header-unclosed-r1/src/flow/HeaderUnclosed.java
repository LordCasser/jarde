package flow;

public class HeaderUnclosed<T> extends GenericParent<T> {
    public @TypeTag T value;
    public java.util.List<@TypeTag T> nested;

    public T echo(T value) { return value; }

    public HeaderUnclosed() {
        super();
    }
}
