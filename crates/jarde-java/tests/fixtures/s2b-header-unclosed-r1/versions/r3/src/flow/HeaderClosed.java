package flow;

public class HeaderClosed<T> extends GenericParent<T> {
    public T value;

    public T echo(T value) { return value; }

    public HeaderClosed() {
        super();
    }
}
