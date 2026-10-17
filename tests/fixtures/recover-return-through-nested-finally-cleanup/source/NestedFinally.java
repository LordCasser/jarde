public class NestedFinally {
    public NestedFinally() {}

    static int nestedFin(int n){ try { return 10/n; } finally { try { System.out.println("in"); } finally { System.out.println("out"); } } }
}
