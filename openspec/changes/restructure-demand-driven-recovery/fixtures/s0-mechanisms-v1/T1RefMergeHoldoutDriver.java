public class T1RefMergeHoldoutDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   T1RefMergeHoldout.trace = "";
   try { System.out.println(n + "|" + T1RefMergeHoldout.run(n) + "|" + T1RefMergeHoldout.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + T1RefMergeHoldout.trace); }
  }
 }
}
