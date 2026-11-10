package flow;

import identity.Foo;
import java.util.List;

public class ReturnConsumptionControls {
    public static List<Foo> returnParameter(List<Foo> input) {
        List<Foo> local = input;
        return local;
    }

    public static <T extends List<Foo>> T echo(T input) {
        return input;
    }

    public static List<Foo> returnField(Base<List<Foo>> input) {
        return input.baseValue;
    }

    @SuppressWarnings("unchecked")
    public static List<Foo> unchecked(Object input) {
        return (List<Foo>) input;
    }
}
