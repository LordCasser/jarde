public class CallReturnFinally {
    static int finReturn(int[] xs){                          // return 穿过 finally（finally 后有副作用）
        for(int x : xs){ try { if(x == 3){ return c0(x); } } finally { System.out.println("fin:" + x); } }
        return -1;
    }
    static int c0(int x){ return x * 1000; }
}
