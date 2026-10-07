import java.util.ArrayList;
import java.util.List;
public class DB {
    static List<String> dbl = new ArrayList<String>() {{ add("a"); add("b"); }};   // 双括号初始化（匿名子类+实例块）
    static int size(){ return dbl.size(); }
    static List<String> withCapture(String s){ return new ArrayList<String>() {{ add(s); }}; }  // 捕获参数
    public static void main(String[] a){ System.out.println(""+size()+"/"+withCapture("z").get(0)); }
}
