import java.util.List;
public class RawBodyMismatch {
    @SuppressWarnings({"rawtypes", "unchecked"})
    public static List body(List values) {
        values.add(Integer.valueOf(42));
        return values;
    }
}
