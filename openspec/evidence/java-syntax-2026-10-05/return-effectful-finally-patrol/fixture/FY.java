public class FY {
    static int emptyFin(int[] xs){                          // return 穿过空 finally（无副作用）
        for(int x : xs){ try { if(x == 3){ return x * 1000; } } finally { } }
        return -1;
    }
    static int noCall(int[] xs){                            // return 穿过副作用 finally（返回简单值非调用）
        int c = 0;
        for(int x : xs){ try { if(x == 3){ return c + 7; } } finally { c += 100; } }
        return c;
    }
    static int loopless(int[] xs){                          // 同形无循环（对照：循环外 return 穿 finally）
        try { if(xs.length > 0 && xs[0] == 3){ return 3000; } } finally { System.out.println("f"); }
        return -1;
    }
    public static void main(String[] a){ System.out.println(""+emptyFin(new int[]{1,3})+"/"+noCall(new int[]{1,3})+"/"+loopless(new int[]{3})); }
}
