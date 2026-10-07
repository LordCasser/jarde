public class CP7 {
    static int scan(int[] xs){                                  // do-while 扫描循环（条件位后缀）
        int i = 0, last = -1;
        do { last = xs[i]; } while (xs[i++] != 0 && i < xs.length);
        return last;
    }
    static int find(int[] xs, int t){                           // while 条件位后缀
        int i = 0;
        while (i < xs.length && xs[i++] != t) { }
        return i - 1;
    }
    static boolean cond(int[] a){                               // if 条件位后缀 + 短路
        int i = 0;
        if (a[i++] > 0 && i < a.length) { return true; }
        return false;
    }
    public static void main(String[] x){ System.out.println(""+scan(new int[]{5,7,0,9})+"/"+find(new int[]{5,7,9},7)+"/"+cond(new int[]{3,1})); }
}
