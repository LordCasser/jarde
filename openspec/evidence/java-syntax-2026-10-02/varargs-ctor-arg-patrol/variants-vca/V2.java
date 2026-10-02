import java.util.ArrayList;
import java.util.Arrays;
public class V2 {
    static int kept;
    static Integer[] keptArr;
    public static int midStatement() { return new ArrayList<Integer>(Arrays.asList(1, kept = 2)).size(); }
    public static int doubleUse() { return new ArrayList<Integer>(Arrays.asList(keptArr = new Integer[]{1, 2})).size(); }
    public static void main(String[] a) {
        System.out.println(midStatement() + ":" + doubleUse() + ":" + kept + ":" + keptArr[0]);
    }
}
