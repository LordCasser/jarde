public class BV {
    static boolean all(boolean a, boolean b){ boolean r = a; r &= b; return r; }        // 布尔 &=
    static boolean any(boolean a, boolean b){ boolean r = a; r |= b; return r; }        // 布尔 |=
    static boolean flip(boolean a){ boolean r = a; r ^= true; return r; }               // 布尔 ^=
    @SafeVarargs
    static <T> int count(T... xs){ int n = 0; for(T x : xs){ if(x != null){ n++; } } return n; }   // SafeVarargs 泛型变参
    static String kind(char c){                                                         // char switch 标签
        switch(c){ case 'a': return "A"; case 'b': case 'c': return "BC"; default: return "?"; }
    }
    public static void main(String[] a){ System.out.println(""+all(true,false)+"/"+all(true,true)+"/"+any(false,true)+"/"+flip(true)+"/"+count("x",null,"y")+"/"+kind('a')+kind('b')+kind('z')); }
}
