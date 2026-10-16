public class LoopTransferFinally {
    static int finContinue(int[] xs){                       // continue 穿过 finally
        int c = 0;
        for(int x : xs){ try { if(x == 3){ continue; } c += x; } finally { c += 100; } }
        return c;
    }
    static int finBreak(int[] xs){                           // break 穿过 finally
        int c = 0;
        for(int x : xs){ try { if(x == 3){ break; } c += x; } finally { c += 100; } }
        return c;
    }
}
