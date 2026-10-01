// 折叠正例矩阵（任务 2.1/2.2）：(byte,String) / getstatic / null / char / boolean / short / int 全形。
// java -Xverify:all 运行基线见 args/runs/args-matrix.original.out；与
// src/enum_constants.rs::arbitrary_argument_shapes_fold_with_narrowed_spellings_and_verified_runtime 同源。
enum BytesText {
    A((byte) 1, "x"), B((byte) 2, "y");
    private final byte num;
    private final String s;
    BytesText(byte n, String s) { this.num = n; this.s = s; }
    byte n() { return num; }
    String text() { return s; }
}
enum CrossRefs {
    R1(BytesText.A), R2(BytesText.B), R3(null);
    private final Object other;
    CrossRefs(Object other) { this.other = other; }
    Object other() { return other; }
}
enum CharArgs {
    ONE('x'), TWO('\''), THREE((char) -1);
    private final char c;
    CharArgs(char c) { this.c = c; }
    char c() { return c; }
}
enum BoolArgs {
    NO(false), YES(true);
    private final boolean flag;
    BoolArgs(boolean flag) { this.flag = flag; }
    boolean flag() { return flag; }
}
enum ShortArgs {
    BIG((short) 300), SMALL((short) -300);
    private final short s;
    ShortArgs(short s) { this.s = s; }
    short s() { return s; }
}
enum IntArgs {
    MID(5), NEG(-1);
    private final int i;
    IntArgs(int i) { this.i = i; }
    int i() { return i; }
}
public class ArgsMatrix {
    public static void main(String[] args) {
        for (BytesText v : BytesText.values()) System.out.println(v.name() + ":" + v.n() + ":" + v.text());
        for (CrossRefs v : CrossRefs.values()) System.out.println(v.name() + ":" + v.other());
        for (CharArgs v : CharArgs.values()) System.out.println(v.name() + ":" + v.c());
        for (BoolArgs v : BoolArgs.values()) System.out.println(v.name() + ":" + v.flag());
        for (ShortArgs v : ShortArgs.values()) System.out.println(v.name() + ":" + v.s());
        for (IntArgs v : IntArgs.values()) System.out.println(v.name() + ":" + v.i());
        System.out.println(BytesText.valueOf("A").text());
        System.out.println(BytesText.A.n() + CrossRefs.R1.other().toString().length());
    }
}
