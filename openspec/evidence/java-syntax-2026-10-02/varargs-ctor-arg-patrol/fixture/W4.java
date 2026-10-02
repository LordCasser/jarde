import java.util.Arrays;
import java.util.List;
public class W4 {
    public static int bare() { return Arrays.asList(1, 2).size(); }
    public static String fmt() { return String.format("%d-%d", 1, 2); }
    public static int assigned() { List<Integer> l = Arrays.asList(5, 6); return l.get(0); }
    public static void main(String[] a) { System.out.println(bare()+":"+fmt()+":"+assigned()); }
}
