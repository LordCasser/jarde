public class R2NestedExitHoldoutDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   R2NestedExitHoldout.trace = "";
   try { System.out.println(n + "|" + R2NestedExitHoldout.run(n) + "|" + R2NestedExitHoldout.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + R2NestedExitHoldout.trace); }
  }
 }
}
