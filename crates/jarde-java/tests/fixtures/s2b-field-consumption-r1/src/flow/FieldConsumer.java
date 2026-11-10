package flow;

import identity.Foo;
import java.util.List;

public final class FieldConsumer {
    public static void typedWrite(FieldBox<List<Foo>> box, List<Foo> next) {
        box.value = next;
    }

    public static void rawWrite(FieldBox<List<Foo>> box, Object next) {
        FieldBox raw = box;
        raw.value = next;
    }

    public static List<Foo> staticRead() {
        return FieldBox.shared;
    }

    public static void staticWrite(List<Foo> next) {
        FieldBox.shared = next;
    }

    public static void primitiveAndLiteral(FieldBox<List<Foo>> box) {
        box.count = 1;
        box.title = "x";
    }

    public static List<Foo> getBaseField(FieldDerived derived) {
        return ((FieldBox<List<Foo>>) derived).value;
    }
}
