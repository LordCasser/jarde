public class C1OverloadedRelayHoldoutDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   C1OverloadedRelayHoldout.trace = "";
   try { System.out.println(n + "|" + C1OverloadedRelayHoldout.run(n) + "|" + C1OverloadedRelayHoldout.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + C1OverloadedRelayHoldout.trace); }
  }
 }
}
