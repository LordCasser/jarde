public class UT extends java.lang.Object {
    static class 内部类 { String 名字 = "中文"; }
    static int 变量;

    static java.lang.String 描述;

    public UT() {
        super();
        return;
    }

    static int 方法(int arg0) {
        return arg0 * 2;
    }

    public static void main(java.lang.String[] arg0) {
        内部类 local1 = new 内部类();
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(描述).append("/").append(方法(21)).append("/").append(local1.名字).toString());
        return;
    }

    static {
        变量 = 1;
        描述 = new java.lang.StringBuilder().append("变量=").append(变量).toString();
    }
}
