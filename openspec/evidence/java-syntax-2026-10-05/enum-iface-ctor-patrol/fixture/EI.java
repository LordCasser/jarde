public class EI {
    interface Op { int apply(int x); }
    enum Basic implements Op {
        PLUS { public int apply(int x){ return x + 1; } },
        MINUS { public int apply(int x){ return x - 1; } };
    }
    enum WithCtor {
        BIG(100), SMALL(1);
        final int scale;
        WithCtor(int s){ scale = s; }
        public int apply(int v){ return v * scale; }
    }
    public static void main(String[] a){ System.out.println(""+Basic.PLUS.apply(5)+"/"+Basic.MINUS.apply(5)+"/"+WithCtor.BIG.apply(3)+"/"+WithCtor.SMALL.apply(3)); }
}
