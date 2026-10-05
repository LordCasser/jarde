public class AN {
    interface Op { int apply(int x); }
    static Op returned(){ return new Op(){ public int apply(int x){ return x + 1; } }; }         // return 位
    static int asArg(int v){ return run(new Op(){ public int apply(int x){ return x * 2; } }, v); }   // 实参位
    static int run(Op o, int v){ return o.apply(v); }
    static Op field = new Op(){ public int apply(int x){ return x - 1; } };                       // 字段初始化位
    static int localVar(){ Op o = new Op(){ public int apply(int x){ return x * x; } }; return o.apply(5); }   // 局部变量位
    public static void main(String[] a){ System.out.println(""+returned().apply(10)+"/"+asArg(21)+"/"+field.apply(10)+"/"+localVar()); }
}
