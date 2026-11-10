public class E2FieldArrayOrderDevDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   E2FieldArrayOrderDev.trace = "";
   try { System.out.println(n + "|" + E2FieldArrayOrderDev.run(n) + "|" + E2FieldArrayOrderDev.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + E2FieldArrayOrderDev.trace); }
  }
 }
}
