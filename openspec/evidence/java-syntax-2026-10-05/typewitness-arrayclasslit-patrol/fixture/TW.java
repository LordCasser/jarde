import java.util.*;
public class TW {
    static List<String> witness(){ return Collections.<String>emptyList(); }                    // 显式类型见证
    static Class<?> arrClass(){ return int[].class; }                                            // 数组类字面量（原始型）
    static Class<?> deepArrClass(){ return String[][].class; }                                   // 二维引用数组类字面量
    enum Color { RED, GREEN, BLUE;
        static String all(){                                                                     // 枚举内 values() 遍历
            StringBuilder sb = new StringBuilder();
            for(Color c : Color.values()){ sb.append(c.name()).append(","); }
            return sb.toString();
        } }
    public static void main(String[] a){ System.out.println(""+witness().size()+"/"+arrClass().getName()+"/"+deepArrClass().getName()+"/"+Color.all()); }
}
