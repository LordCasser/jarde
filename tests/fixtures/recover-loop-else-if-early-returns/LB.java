public class LB {
    // 本片（recover-loop-else-if-early-returns）的负例与边界形：全部**保持拒绝**，
    // 每形一条可重放的 class 字节（双腿），由 tests/recover_loop_else_if_early_returns.rs 钉住。

    static int doubleLadder(int[] xs, int k){                   // 双层阶梯（MVP 外形：非单层）
        int i = 0;
        while(i < xs.length){ int v = xs[i];
            if(v < k){ i = i + 1; }
            else if(v > k){ i = i + 2; }
            else if(v == k){ return i; }
            else { i = i + 3; } }
        return -1;
    }
    static int tryLadder(int[] xs, int k){                      // 异常表跨越阶梯（MVP 外形）
        int lo = 0, hi = xs.length - 1;
        while(lo <= hi){ int m = (lo + hi) >>> 1;
            try { int v = xs[m];
                if(v < k){ lo = m + 1; } else if(v > k){ hi = m - 1; } else { return m; } }
            catch(ArrayIndexOutOfBoundsException e){ return -2; } }
        return -1;
    }
    static int switchLadder(int[] xs, int k){                   // switch 与阶梯同级（本片读法仍恢复，见 evidence）
        int i = 0, r = 0;
        while(i < xs.length){ int v = xs[i];
            switch(v){ case 0: r = r + 1; break; default: r = r - 1; }
            if(v < k){ i = i + 1; } else if(v > k){ i = i + 2; } else { return r; } }
        return r;
    }
    static int switchInArm(int[] xs, int k){                    // switch 混入阶梯臂（MVP 外形）
        int i = 0, r = 0;
        while(i < xs.length){ int v = xs[i];
            if(v < k){ i = i + 1; }
            else if(v > k){ switch(v % 2){ case 0: r = r + 1; break; default: r = r - 1; } i = i + 2; }
            else { return r; } }
        return r;
    }
    static int firstArmRet(int[] xs, int k){                    // 早退在第一臂（本片读法未覆盖）
        int lo = 0, hi = xs.length - 1;
        while(lo <= hi){ int m = (lo + hi) >>> 1; int v = xs[m];
            if(v == k){ return m; } else if(v < k){ lo = m + 1; } else { hi = m - 1; } }
        return -1;
    }
    static int forLadder(int[] xs, int k){                      // for 头 + 阶梯早退（探针）
        for(int i = 0; i < xs.length; i++){
            int v = xs[i];
            if(v < k){ i = i + 1; } else if(v > k){ i = i + 2; } else { return i; } }
        return -1;
    }
    public static void main(String[] a){
        System.out.println(""+doubleLadder(new int[]{1,2,3}, 2)+"/"+tryLadder(new int[]{1,2,3}, 2)
            +"/"+switchLadder(new int[]{0,1,0}, 1)+"/"+switchInArm(new int[]{1,3,5}, 3)
            +"/"+firstArmRet(new int[]{1,3,5}, 3)+"/"+forLadder(new int[]{1,3,5}, 3));
    }
}
