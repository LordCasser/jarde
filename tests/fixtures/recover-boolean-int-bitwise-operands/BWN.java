public class BWN {
    static int plusOne(boolean b, int x){ return (b ? 1 : 0) + x; }
    static int andNotInt(boolean a, boolean b){ return (a & !b) ? 1 : 0; }
    static boolean compareRead(boolean[] f, boolean other){ boolean r = false; for(boolean x : f) r ^= x; return r == other; }
    static int intSibling(int r, boolean b){ return r ^ (b ? 1 : 0); }
    static void passed(boolean[] f){ boolean r = false; for(boolean x : f) r ^= x; java.lang.System.out.println(r); }
    public static void main(String[] a){ System.out.println(""+plusOne(true,5)+"/"+andNotInt(true,true)+"/"+compareRead(new boolean[]{true,true},true)+"/"+intSibling(6,true)); passed(new boolean[]{true,false}); }
}
