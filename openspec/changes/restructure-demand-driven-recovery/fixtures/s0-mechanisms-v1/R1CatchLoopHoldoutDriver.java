public class R1CatchLoopHoldoutDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   R1CatchLoopHoldout.trace = "";
   try { System.out.println(n + "|" + R1CatchLoopHoldout.run(n) + "|" + R1CatchLoopHoldout.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + R1CatchLoopHoldout.trace); }
  }
 }
}
