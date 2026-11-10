public class C2AnonymousFactoryHoldoutDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   C2AnonymousFactoryHoldout.trace = "";
   try { System.out.println(n + "|" + C2AnonymousFactoryHoldout.run(n) + "|" + C2AnonymousFactoryHoldout.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + C2AnonymousFactoryHoldout.trace); }
  }
 }
}
