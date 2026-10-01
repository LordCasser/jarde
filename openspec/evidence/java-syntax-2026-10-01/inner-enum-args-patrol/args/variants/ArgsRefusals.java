// 五个 verifier 有效的拒绝形态（任务 1.2）：逐字段呈现保持；各自 java -Xverify:all 通过。
enum KindMismatchObject {
    A("plain");                       // 实参种类不符：Object 参收到 String 字面量（期望 getstatic|null）
    private final Object v;
    KindMismatchObject(Object v) { this.v = v; }
    Object v() { return v; }
}
enum StringParamGetstatic {
    A(StrHold.source);                // 实参种类不符：String 参收到 getstatic（期望 ldc 字面量）
    private final String v;
    StringParamGetstatic(String v) { this.v = v; }
    String v() { return v; }
}
enum ExtraStatement {
    A((byte) 1, "x");                 // ctor 体额外语句（println 在存字段之前）
    private final byte num;
    private final String s;
    ExtraStatement(byte n, String s) { System.out.println("side"); this.num = n; this.s = s; }
    byte n() { return num; }
}
enum TooManyArgs {
    A((byte) 1, "x", java.math.BigInteger.ZERO, 5);   // >3 用户参
    private final byte b;
    private final String s;
    private final java.math.BigInteger big;
    private final int i;
    TooManyArgs(byte b, String s, java.math.BigInteger big, int i) {
        this.b = b; this.s = s; this.big = big; this.i = i;
    }
    byte b() { return b; }
    String s() { return s; }
    java.math.BigInteger big() { return big; }
    int i() { return i; }
}
enum LongArg {
    A(1L);                            // long 形态不做（J/F/D 与数组不在 grammar），登记为未做
    private final long v;
    LongArg(long v) { this.v = v; }
    long v() { return v; }
}
class StrHold {
    static String source = "held";
}
public class ArgsRefusals {
    public static void main(String[] args) {
        System.out.println(KindMismatchObject.A.v());
        System.out.println(StringParamGetstatic.A.v());
        System.out.println(ExtraStatement.A.n());
        System.out.println(TooManyArgs.A.b() + ":" + TooManyArgs.A.s() + ":" + TooManyArgs.A.big() + ":" + TooManyArgs.A.i());
        System.out.println(LongArg.A.v());
    }
}
