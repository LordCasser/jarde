package cf11;
import java.util.Arrays;
import java.util.List;
public final class Runner {
    private static String repeated(char value, int count) {
        char[] chars = new char[count];
        Arrays.fill(chars, value);
        return new String(chars);
    }
    private static String outcome(UpstreamNestedLoops2 subject, List<String> values) {
        try {
            return Boolean.toString(subject.test(values));
        } catch (NullPointerException error) {
            return error.getClass().getName();
        }
    }
    private static void emit(String name, String result) {
        System.out.println(name + "=" + result);
    }
    public static void main(String[] args) {
        UpstreamNestedLoops2 subject = new UpstreamNestedLoops2();
        emit("empty", outcome(subject, Arrays.<String>asList()));
        emit("same6", outcome(subject, Arrays.asList(repeated('x', 6), repeated('y', 6))));
        emit("ten", outcome(subject, Arrays.asList(repeated('x', 10), repeated('y', 10))));
        emit("eleven_then_zero", outcome(subject, Arrays.asList(repeated('x', 11), "")));
        emit("zero_then_eleven", outcome(subject, Arrays.asList("", repeated('x', 11))));
        emit("ascending", outcome(subject, Arrays.asList(repeated('x', 3), repeated('y', 11))));
        emit("descending", outcome(subject, Arrays.asList(repeated('x', 11), repeated('y', 3))));
        emit("null_list", outcome(subject, null));
        emit("null_element", outcome(subject, Arrays.<String>asList((String) null)));
    }
}
