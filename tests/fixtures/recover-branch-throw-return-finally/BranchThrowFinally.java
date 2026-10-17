public class BranchThrowFinally {
    static String finThrow(int n){ try { if(n<0) throw new RuntimeException("neg"); return "ok"; } finally { System.out.println("ft"); } }
}
