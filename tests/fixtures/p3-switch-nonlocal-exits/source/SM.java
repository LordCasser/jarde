public class SM {
    static int onlyBreak(int[] xs){ int sum = 0; for(int x : xs){ switch(x){ case 1: break; default: sum += x; } sum += 10; } return sum; }   // 只有 break（switch 退出到循环尾）
    static int onlyContinue(int[] xs){ int sum = 0; for(int x : xs){ switch(x){ case 2: continue; default: sum += x; } } return sum; }       // 只有 continue（无循环尾语句）
    static int breakNoTail(int[] xs){ int sum = 0; for(int x : xs){ switch(x){ case 1: break; default: sum += x; } } return sum; }           // break 但循环尾为空（break 与 continue 不可区分）
    static int ifVersion(int[] xs){ int sum = 0; for(int x : xs){ if(x == 1){ } else { sum += x; } sum += 10; } return sum; }                 // if/else 对照（同结构无 switch）
    static int noTailMix(int[] xs){ int sum = 0; for(int x : xs){ switch(x){ case 1: break; case 2: continue; default: sum += x; } } return sum; }  // break+continue 但无循环尾
    public static void main(String[] a){ System.out.println(""+onlyBreak(new int[]{1,4})+"/"+onlyContinue(new int[]{2,4})+"/"+breakNoTail(new int[]{1,4})+"/"+ifVersion(new int[]{1,4})+"/"+noTailMix(new int[]{1,2,4})); }
}
