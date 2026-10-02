import java.util.ArrayList;
import java.util.Arrays;
public class V3 {
    public static int viaMixed() { return new ArrayList<Number>(Arrays.asList(1, 2L, 3.0)).size(); }
    public static void main(String[] a) { System.out.println(viaMixed()); }
}
