public class E1ArrayIndexPostfixHoldoutDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   E1ArrayIndexPostfixHoldout.trace = "";
   try { System.out.println(n + "|" + E1ArrayIndexPostfixHoldout.run(n) + "|" + E1ArrayIndexPostfixHoldout.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + E1ArrayIndexPostfixHoldout.trace); }
  }
 }
}
