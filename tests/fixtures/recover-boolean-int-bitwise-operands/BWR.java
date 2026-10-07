public class BWR {
    static boolean accXor(boolean[] f, boolean c){ boolean r = false; for(boolean x : f) r ^= x; return r ^ c; }
    static boolean andNotAnd(boolean a, boolean b, boolean c){ return (a & !b) & c; }
    static boolean orNot(boolean a, boolean b){ return a | !b; }
    static boolean notOr(boolean a, boolean b){ return !a | b; }
    public static void main(String[] a){ System.out.println(""+accXor(new boolean[]{true,false,true},false)+"/"+andNotAnd(true,true,true)+"/"+orNot(false,true)+"/"+notOr(true,false)); }
}
