public class SN {
    static int retInSwitchNoTail(int[] xs){ int sum = 0; for(int x : xs){ switch(x){ case 3: return sum + 100; default: sum += x; } } return sum; }   // return 在 case、无循环尾
    static int retInSwitchTail(int[] xs){ int sum = 0; for(int x : xs){ switch(x){ case 3: return sum + 100; default: sum += x; } sum += 10; } return sum; }  // return 在 case、有循环尾
    static int retAfterTail(int[] xs){ int sum = 0; for(int x : xs){ if(x == 3){ return sum + 100; } sum += x; sum += 10; } return sum; }  // if 对照（return+尾）
    static int labBreak(int[] xs){ int sum = 0; outer: for(int x : xs){ switch(x){ case 5: break outer; default: sum += x; } } return sum; }   // 单独 labeled break
    public static void main(String[] a){ System.out.println(""+retInSwitchNoTail(new int[]{1,3})+"/"+retInSwitchTail(new int[]{1,3})+"/"+retAfterTail(new int[]{1,3})+"/"+labBreak(new int[]{1,5})); }
}
