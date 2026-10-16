public class NoCallFinally {
    static int noCall(int[] xs){                            // return 穿过副作用 finally（返回简单值非调用）
        int c = 0;
        for(int x : xs){ try { if(x == 3){ return c + 7; } } finally { c += 100; } }
        return c;
    }
}
