public class UT {                                              // unicode 标识符族（untested 维度）
    static int 变量 = 1;
    static int 方法(int 参数){ return 参数 * 2; }
    static class 内部类 { String 名字 = "中文"; }
    static String 描述 = "变量=" + 变量;
    public static void main(String[] a){ 内部类 o = new 内部类(); System.out.println(描述 + "/" + 方法(21) + "/" + o.名字); }
}
