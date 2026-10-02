import java.util.ArrayList;
import java.util.Arrays;
public class W3 {
    public static int viaArrays() { return new ArrayList<Integer>(Arrays.asList(1, 2, 3)).size(); }
    public static int viaArraysEmpty() { return new ArrayList<Integer>(Arrays.asList(new Integer[0])).size(); }
    public static String viaClassLit() { return nameOf(W3.class); }
    static String nameOf(Class<?> c) { return c.getSimpleName(); }
    public static void main(String[] a) { System.out.println(viaArrays()+":"+viaArraysEmpty()+":"+viaClassLit()); }
}
