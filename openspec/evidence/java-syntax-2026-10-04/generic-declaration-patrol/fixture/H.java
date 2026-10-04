import java.util.*;
public class H {
    static List<String> unread = new ArrayList<String>();     // 从不被读
    static List<String> readInMain = new ArrayList<String>(); // 被 main 读（getstatic）
    static List<String> initDiamond = new ArrayList<>();      // diamond 初始化，从不被读
    public static void main(String[] a) {
        System.out.println(readInMain.size());
    }
}
