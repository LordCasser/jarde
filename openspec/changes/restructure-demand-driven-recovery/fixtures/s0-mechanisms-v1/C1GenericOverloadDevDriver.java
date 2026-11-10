public class C1GenericOverloadDevDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   C1GenericOverloadDev.trace = "";
   try { System.out.println(n + "|" + C1GenericOverloadDev.run(n) + "|" + C1GenericOverloadDev.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + C1GenericOverloadDev.trace); }
  }
 }
}
