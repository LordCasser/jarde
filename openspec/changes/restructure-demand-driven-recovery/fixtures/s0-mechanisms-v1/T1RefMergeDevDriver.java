public class T1RefMergeDevDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   T1RefMergeDev.trace = "";
   try { System.out.println(n + "|" + T1RefMergeDev.run(n) + "|" + T1RefMergeDev.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + T1RefMergeDev.trace); }
  }
 }
}
