package flow;

import identity.Foo;
import java.util.List;

public class CallAdaptationControls {
    public static Foo typedAdd(List<Foo> input, Foo value, int index) {
        List<Foo> local = input;
        local.add(value);
        return local.get(index);
    }

    public static void genericForward(List<Foo> source, List<Foo> target, int index) {
        target.add(source.get(index));
    }

    @SuppressWarnings("unchecked")
    public static void rawWrite(List<Foo> input, Object value) {
        List raw = input;
        raw.add(value);
    }
}
