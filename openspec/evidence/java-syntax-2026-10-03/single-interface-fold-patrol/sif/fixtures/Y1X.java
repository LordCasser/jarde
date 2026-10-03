import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
public class Y1X {
    interface StrFn { String apply(String s); }
    static class Impl implements StrFn { public String apply(String s) { return s + "!"; } }
    static String viaImpl(String s) { return new Impl().apply(s); }
    static int viaLen() { return 42 + "hey".length(); }
    static List<String> viaList() {
        List<String> xs = new ArrayList<String>(Arrays.asList("b", "aa", "ccc"));
        xs.remove("ccc");
        return xs;
    }
    static int plusN(int n, int v) { return v + n; }
    static int captureUse(int n) { return plusN(n, 5); }
    public static void main(String[] a) {
        System.out.println(viaImpl("hi"));
        System.out.println(viaLen());
        System.out.println(viaList());
        System.out.println(captureUse(3));
    }
}
