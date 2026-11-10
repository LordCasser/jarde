public class R1LoopCatchDevDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   R1LoopCatchDev.trace = "";
   try { System.out.println(n + "|" + R1LoopCatchDev.run(n) + "|" + R1LoopCatchDev.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + R1LoopCatchDev.trace); }
  }
 }
}
