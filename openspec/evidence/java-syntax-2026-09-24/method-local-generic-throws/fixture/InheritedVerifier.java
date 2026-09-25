package methodthrows;

public final class InheritedVerifier {
    public static void main(String[] args) throws Exception {
        System.out.println("loaded=" + Class.forName("methodthrows.InheritedMethodBoundary").getName());
    }
}
