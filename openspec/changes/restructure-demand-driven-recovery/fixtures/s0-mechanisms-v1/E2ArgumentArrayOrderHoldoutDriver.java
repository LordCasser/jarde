public class E2ArgumentArrayOrderHoldoutDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   E2ArgumentArrayOrderHoldout.trace = "";
   try { System.out.println(n + "|" + E2ArgumentArrayOrderHoldout.run(n) + "|" + E2ArgumentArrayOrderHoldout.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + E2ArgumentArrayOrderHoldout.trace); }
  }
 }
}
