import java.util.*;
public class G1 {
    public static List<String> paramEcho(List<String> in) { return in; }        // 参数→返回（DT-18 已验收）
    public static List<String> nullRet() { return null; }                        // null 返回（null-return 片）
    public static List<String> newRet() { return new ArrayList<String>(); }      // new 返回 ← 本探针
    public static List<String> fieldRet() { return F; }                          // 字段读返回
    static List<String> F = new ArrayList<String>();
    public void main(String[] a) {}
}
