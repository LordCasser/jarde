/** Modern string concat: what javac compiles `+` into from JDK 9 on. */
public class ConcatSample {
    public static String mixed(int count, Object value) {
        return "count=" + count + " value=" + value + ".";
    }

    public static String pair(String left, String right) {
        return left + right;
    }
}
