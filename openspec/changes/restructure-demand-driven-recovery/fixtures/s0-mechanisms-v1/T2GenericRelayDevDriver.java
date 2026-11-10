public class T2GenericRelayDevDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   T2GenericRelayDev.trace = "";
   try { System.out.println(n + "|" + T2GenericRelayDev.run(n) + "|" + T2GenericRelayDev.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + T2GenericRelayDev.trace); }
  }
 }
}
