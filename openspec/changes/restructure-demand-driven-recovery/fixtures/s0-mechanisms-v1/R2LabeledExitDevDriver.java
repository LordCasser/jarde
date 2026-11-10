public class R2LabeledExitDevDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   R2LabeledExitDev.trace = "";
   try { System.out.println(n + "|" + R2LabeledExitDev.run(n) + "|" + R2LabeledExitDev.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + R2LabeledExitDev.trace); }
  }
 }
}
