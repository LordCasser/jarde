import java.util.*;
public class AH {
    static List<String> byTop(List<String> in){
        List<String> out = new ArrayList<String>(in);
        Collections.sort(out, new AC());                     // 与 CP.byAnon 同形、实参类型无 $
        return out;
    }
    public static void main(String[] a){ System.out.println(byTop(new ArrayList<String>(Arrays.asList("bb","a","ccc")))); }
}
