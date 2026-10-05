public class FX {
    static int finContinue(int[] xs){                       // continue 穿过 finally
        int c = 0;
        for(int x : xs){ try { if(x == 3){ continue; } c += x; } finally { c += 100; } }
        return c;
    }
    static int finReturn(int[] xs){                          // return 穿过 finally（finally 后有副作用）
        for(int x : xs){ try { if(x == 3){ return c0(x); } } finally { System.out.println("fin:" + x); } }
        return -1;
    }
    static int c0(int x){ return x * 1000; }
    static int finBreak(int[] xs){                           // break 穿过 finally
        int c = 0;
        for(int x : xs){ try { if(x == 3){ break; } c += x; } finally { c += 100; } }
        return c;
    }
    public static void main(String[] a){ System.out.println(""+finContinue(new int[]{3,4})+"/"+finReturn(new int[]{1,3,5})+"/"+finBreak(new int[]{3,4})); }
}
