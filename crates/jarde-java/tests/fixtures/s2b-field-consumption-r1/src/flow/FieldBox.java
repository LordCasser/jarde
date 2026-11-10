package flow;

import identity.Foo;
import java.util.List;

public class FieldBox<T> {
    public T value;
    public int count;
    public String title;
    public static List<Foo> shared;
}
