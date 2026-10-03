public class RF {
    // 直属成员含 <clinit>（静态初始化）-> 触发 "member fold ... initializer projection this fold does not carry"
    static class Inner { static final int K = init(); static int init() { return 7; } }
    public static String simpleName() { return Inner.class.getSimpleName(); }
    public static void main(String[] a) { System.out.println(simpleName()); System.out.println(Inner.K); }
}
