public class FW {
    // 内部类写外类 final 字段（写形访问器 + final 语义）
    private final int fin;
    private int mut = 5;
    FW(){ fin = 10; }
    class W {
        int rd(){ return fin; }               // 读 final（ctor 已定，无写访问器）
        int wr(int v){ mut = v; return mut; } // 读写混合
        int rdFin2(){ return FW.this.fin; }   // 显式外类限定读
    }
    static int statW = 0;
    class SW { void w(){ statW = 9; } }       // 实例内部类写外类静态
    public static void main(String[] a){ FW f = new FW(); W w = f.new W(); SW s = f.new SW(); s.w(); System.out.println(""+w.rd()+"/"+w.wr(3)+"/"+w.rdFin2()+"/"+statW); }
}
