public class EC {
    static int calls = 0;
    static boolean bump(){ EC.calls++; return EC.calls < 3; }
    static int emptyThen(int x){ if(x > 0) {} return x; }                    // 空 then
    static int emptyBoth(int x){ if(x > 0) {} else {} return x; }            // 双空
    static int emptyLoop(){ while(bump()) {} return calls; }                 // 空循环体+副作用条件
    static int emptyFor(){ for(int i = 0; i < 3; i++) {} return 9; }         // 空 for 体
    static int emptyTry(){ try {} finally { return 5; } }                    // 空 try+finally return
    public static void main(String[] a){ System.out.println(""+emptyThen(1)+"/"+emptyBoth(2)+"/"+emptyLoop()+"/"+emptyFor()+"/"+emptyTry()); }
}
