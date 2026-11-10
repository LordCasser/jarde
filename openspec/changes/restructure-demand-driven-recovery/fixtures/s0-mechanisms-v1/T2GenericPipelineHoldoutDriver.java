public class T2GenericPipelineHoldoutDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   T2GenericPipelineHoldout.trace = "";
   try { System.out.println(n + "|" + T2GenericPipelineHoldout.run(n) + "|" + T2GenericPipelineHoldout.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + T2GenericPipelineHoldout.trace); }
  }
 }
}
