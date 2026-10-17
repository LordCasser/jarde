public class SL {
    static int mixed(int[] xs){                        // break(switch)/continue(loop)/return(method) 混用
        int sum = 0;
        for(int x : xs){
            switch(x){
                case 1: break;                          // 只退出 switch——继续循环体尾
                case 2: continue;                       // 跳到下一迭代
                case 3: return sum + 100;               // 退出方法
                default: sum += x;                      // 落穿无
            }
            sum += 10;                                  // 循环体尾（case 1 的 break 到这里；case 2 不经过）
        }
        return sum;
    }
    static int labeledBreak(int[] xs){                  // 带标签 break 从 switch 内跳出循环
        int sum = 0;
        outer:
        for(int x : xs){
            switch(x){
                case 5: break outer;                    // 从 switch 内直接退出循环
                default: sum += x;
            }
        }
        return sum;
    }
    public static void main(String[] a){ System.out.println(""+mixed(new int[]{1,2,4})+"/"+mixed(new int[]{1,4})+"/"+labeledBreak(new int[]{1,5,9})); }
}
