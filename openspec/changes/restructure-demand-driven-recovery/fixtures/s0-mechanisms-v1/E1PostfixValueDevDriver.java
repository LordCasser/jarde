public class E1PostfixValueDevDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   E1PostfixValueDev.trace = "";
   try { System.out.println(n + "|" + E1PostfixValueDev.run(n) + "|" + E1PostfixValueDev.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + E1PostfixValueDev.trace); }
  }
 }
}
