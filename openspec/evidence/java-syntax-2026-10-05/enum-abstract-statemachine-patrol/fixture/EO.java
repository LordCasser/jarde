public enum EO {                                              // 抽象方法强制逐常量覆写（状态机经典形）
    ADD { int apply(int a, int b){ return a + b; } },
    SUB { int apply(int a, int b){ return a - b; } },
    IDENT { int apply(int a, int b){ return a; } };
    abstract int apply(int a, int b);
    static int run(EO op, int a, int b){ return op.apply(a, b); }   // 经调用点分派
    public static void main(String[] args){ System.out.println(""+run(ADD, 5, 3)+"/"+run(SUB, 5, 3)+"/"+run(IDENT, 5, 3)+"/"+valueOf("SUB")); }
}
