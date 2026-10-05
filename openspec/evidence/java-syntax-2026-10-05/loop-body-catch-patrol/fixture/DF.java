public class DF {
    static int defaultFirst(int x){                 // default 在首位 + 无贯穿（语义等价检查）
        switch(x){ default: return 0; case 1: return 10; case 2: return 20; }
    }
    static int defaultFallthrough(int x){           // default 在中位 + 贯穿（顺序有语义影响！）
        switch(x){ case 1: return 100; default: case 2: return 200; }   // default 贯穿到 case 2
    }
    static int loopCatch(int[] xs){                 // 循环内 try-catch（每迭代捕获继续）
        int good = 0;
        for(int x : xs){ try { if(x < 0){ throw new IllegalArgumentException("neg"); } good += x; } catch(IllegalArgumentException e){ good -= 1; } }
        return good;
    }
    static int loopFinally(int[] xs){               // 循环内 try-finally（每迭代 finally）
        int c = 0;
        for(int x : xs){ try { if(x == 3){ continue; } c += x; } finally { c += 1000; } }
        return c;
    }
    public static void main(String[] a){ System.out.println(""+defaultFirst(1)+"/"+defaultFirst(9)+"/"+defaultFallthrough(1)+"/"+defaultFallthrough(2)+"/"+defaultFallthrough(9)+"/"+loopCatch(new int[]{1,-2,3})+"/"+loopFinally(new int[]{3,4})); }
}
