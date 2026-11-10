package flow;

import java.util.List;
import identity.Foo;

public class BoxConsumer {
    public static Object rawValue(Box box) {
        return box.value;
    }

    public static Object inheritedRawValue(RawChildBox box) {
        return box.value;
    }

    public static String inheritedStringValue(StringChildBox box) {
        return box.value;
    }

    public static Object rawFixed(Box box) {
        return box.fixed;
    }

    public static Object rawEcho(Box box, Object input) {
        return box.echo(input);
    }

    public static List<String> parameterizedFixed(Box<Foo> box) {
        return box.fixed;
    }

    public static Object parameterizedEcho(Box<Foo> box, Object input) {
        return box.echo(input);
    }

    public static List<String> staticField() {
        return Box.fixedStatic;
    }

    public static List<String> rawQualifierStaticField(Box box) {
        return box.fixedStatic;
    }

    public static List<String> typedQualifierStaticField(Box<Foo> box) {
        return box.fixedStatic;
    }

    public static List<String> staticPass(List<String> input) {
        return Box.passthrough(input);
    }

    public static Object staticEcho(Object input) {
        return Box.staticEcho(input);
    }
}
