package flow;

import java.util.List;

public class Box<T> {
    public T value;
    public List<String> fixed;
    public static List<String> fixedStatic;

    public <U> U echo(U input) {
        return input;
    }

    public static List<String> passthrough(List<String> input) {
        return input;
    }

    public static <U> U staticEcho(U input) {
        return input;
    }
}

class RawChildBox extends Box {
}

class StringChildBox extends Box<String> {
}
