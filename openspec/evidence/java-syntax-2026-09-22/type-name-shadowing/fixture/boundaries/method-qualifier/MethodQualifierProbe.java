import java.util.function.Function;

public class MethodQualifierProbe {
    public static Function<String, Integer> ref(MethodQualifierOther value) {
        return arg0::pick;
    }
}
