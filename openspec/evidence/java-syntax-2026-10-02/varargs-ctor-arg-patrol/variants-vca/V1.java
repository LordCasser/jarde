import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashSet;
public class V1 {
    public static int viaEmptyCall() { return new ArrayList<String>(Arrays.asList()).size(); }
    public static int viaBoxedMix() { return new ArrayList<Integer>(Arrays.asList(1, Integer.valueOf(2), 3)).size(); }
    public static int viaHashSet() { return new HashSet<String>(Arrays.asList("a", "b")).size(); }
    public static int viaCallElements() { return new ArrayList<Integer>(Arrays.asList(Integer.valueOf(7), box(8))).size(); }
    static Integer box(int v) { return v; }
    public static void main(String[] a) {
        System.out.println(viaEmptyCall() + ":" + viaBoxedMix() + ":" + viaHashSet() + ":" + viaCallElements());
    }
}
